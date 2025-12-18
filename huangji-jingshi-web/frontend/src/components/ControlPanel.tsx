
import React, { useState, useEffect } from 'react';

interface ControlPanelProps {
  initialParams: {
    datetime: string;
    lat: number;
    lon: number;
  };
  onCalculate: (params: { datetime: string; lat: number; lon: number }) => void;
  isLoading: boolean;
}

// P1 修复：固定使用 UTC+8 (北京时间)，不依赖浏览器时区
const TZ_OFFSET_MINUTES = 480; // UTC+8 = 480 分钟

const ControlPanel: React.FC<ControlPanelProps> = ({ initialParams, onCalculate, isLoading }) => {
  // P1 修复：显式时区转换函数
  // 将本地时间字符串 "YYYY-MM-DDTHH:mm" 按 UTC+8 解析为 UTC ISO 字符串
  const localToUtcIso = (localDatetime: string): string => {
    if (!localDatetime) return new Date().toISOString();
    const [datePart, timePart] = localDatetime.split('T');
    const [y, m, d] = datePart.split('-').map(Number);
    const [hh, mm] = timePart.split(':').map(Number);
    // 构造 UTC 时间戳：本地时间 - 时区偏移
    const utcMs = Date.UTC(y, m - 1, d, hh, mm) - TZ_OFFSET_MINUTES * 60_000;
    return new Date(utcMs).toISOString();
  };

  // 将 UTC ISO 字符串转为本地时间字符串 (UTC+8)
  const utcIsoToLocal = (isoString: string): string => {
    const utcDate = new Date(isoString);
    const localMs = utcDate.getTime() + TZ_OFFSET_MINUTES * 60_000;
    const localDate = new Date(localMs);
    return localDate.toISOString().slice(0, 16);
  };

  // 获取当前北京时间的本地格式
  const getCurrentLocalISO = () => {
    const now = new Date();
    const localMs = now.getTime() + TZ_OFFSET_MINUTES * 60_000;
    const localDate = new Date(localMs);
    return localDate.toISOString().slice(0, 16);
  };

  const initialLive = (() => {
    const paramDate = new Date(initialParams.datetime);
    const now = new Date();
    const diff = Math.abs(paramDate.getTime() - now.getTime());
    return diff <= 60000;
  })();
  const initialDatetime = (() => {
    if (!initialLive) {
      return utcIsoToLocal(initialParams.datetime);
    }
    return getCurrentLocalISO();
  })();
  const [localDatetime, setLocalDatetime] = useState(initialDatetime);
  const [isLive, setIsLive] = useState(initialLive);
  const [lat, setLat] = useState(initialParams.lat.toString());
  const [lon, setLon] = useState(initialParams.lon.toString());
  const [error, setError] = useState<string | null>(null);
  const [locationStatus, setLocationStatus] = useState<string>("");

  // Helper retained above

  // Live clock effect
  useEffect(() => {
    if (!isLive) return;
    const timer = setInterval(() => {
      setLocalDatetime(getCurrentLocalISO());
    }, 1000);
    return () => clearInterval(timer);
  }, [isLive]);

  // Initialize local state from props only on first mount if needed, 
  // but actually we want to default to live mode, so we might just ignore initialParams.datetime 
  // if we want strict "live by default". 
  // However, if user refreshes with a specific URL param, we should probably respect that and turn off live mode?
  // Let's decide: If initialParams.datetime is significantly different from now, assume locked.
  // Preloaded from props in initial state above; no effect needed

  // IP 定位回退
  const getLocationByIP = async (): Promise<{ lat: number; lon: number; source: string } | null> => {
    const apis = [
      { url: 'https://ipapi.co/json/', parse: (d: { latitude?: number; longitude?: number }) => d.latitude && d.longitude ? { lat: d.latitude, lon: d.longitude } : null },
      { url: 'https://ip-api.com/json/?fields=lat,lon', parse: (d: { lat?: number; lon?: number }) => d.lat && d.lon ? { lat: d.lat, lon: d.lon } : null },
      { url: 'https://ipwho.is/', parse: (d: { latitude?: number; longitude?: number }) => d.latitude && d.longitude ? { lat: d.latitude, lon: d.longitude } : null },
    ];
    for (const api of apis) {
      try {
        const resp = await fetch(api.url, { timeout: 5000 } as RequestInit);
        if (resp.ok) {
          const data = await resp.json();
          const loc = api.parse(data);
          if (loc) return { ...loc, source: 'IP' };
        }
      } catch { /* 继续尝试下一个 */ }
    }
    return null;
  };

  // 更新位置并自动联动星空
  const updateLocationAndSync = (newLat: number, newLon: number, source: string) => {
    setLat(newLat.toFixed(4));
    setLon(newLon.toFixed(4));
    setLocationStatus(`${source}已锁定`);
    
    // P1 修复：使用显式时区转换
    onCalculate({
      datetime: localToUtcIso(localDatetime),
      lat: newLat,
      lon: newLon
    });
    
    setTimeout(() => setLocationStatus(""), 5000);
  };

  const handleGeolocation = async () => {
    setLocationStatus("GPS定位中...");
    
    // 首先尝试 GPS/BDS 定位
    if (navigator.geolocation) {
      const gpsPromise = new Promise<{ lat: number; lon: number } | null>((resolve) => {
        navigator.geolocation.getCurrentPosition(
          (position) => {
            resolve({ lat: position.coords.latitude, lon: position.coords.longitude });
          },
          () => {
            resolve(null); // GPS 失败，返回 null
          },
          { timeout: 8000, enableHighAccuracy: true, maximumAge: 60000 }
        );
      });

      const gpsResult = await gpsPromise;
      if (gpsResult) {
        updateLocationAndSync(gpsResult.lat, gpsResult.lon, 'GPS');
        return;
      }
    }

    // GPS 失败，尝试 IP 定位
    setLocationStatus("IP定位中...");
    const ipResult = await getLocationByIP();
    if (ipResult) {
      updateLocationAndSync(ipResult.lat, ipResult.lon, 'IP');
      return;
    }

    // 两种方式都失败
    setLocationStatus("定位失败");
    setTimeout(() => setLocationStatus(""), 3000);
  };
  
  // 页面加载时自动尝试定位
  useEffect(() => {
    // 延迟自动定位，避免阻塞页面加载
    const timer = setTimeout(() => {
      handleGeolocation();
    }, 1000);
    return () => clearTimeout(timer);
  // eslint-disable-next-line react-hooks/exhaustive-deps
  }, []);

  const handleSubmit = () => {
    // Validation
    const latNum = parseFloat(lat);
    const lonNum = parseFloat(lon);

    if (isNaN(latNum) || latNum < -90 || latNum > 90) {
      setError("纬度必须在 -90 到 90 之间");
      return;
    }
    if (isNaN(lonNum) || lonNum < -180 || lonNum > 180) {
      setError("经度必须在 -180 到 180 之间");
      return;
    }

    setError(null);

    // P1 修复：使用显式 UTC+8 转换，不依赖浏览器时区
    onCalculate({
      datetime: localToUtcIso(localDatetime),
      lat: latNum,
      lon: lonNum
    });
  };

  // P1 修复：固定显示 UTC+8，与实际转换逻辑一致
  const offsetString = 'UTC+8';

  const handleSetNow = () => {
    setIsLive(true); // Re-enable live mode
  };

  const lastSubmitRef = React.useRef(0);
  useEffect(() => {
    if (!isLive) return;
    if (!localDatetime) return;
    const now = Date.now();
    if (now - lastSubmitRef.current < 3000) return;
    lastSubmitRef.current = now;
    const latNum = parseFloat(lat);
    const lonNum = parseFloat(lon);
    if (isNaN(latNum) || latNum < -90 || latNum > 90) return;
    if (isNaN(lonNum) || lonNum < -180 || lonNum > 180) return;
    // P1 修复：使用显式 UTC+8 转换
    onCalculate({ datetime: localToUtcIso(localDatetime), lat: latNum, lon: lonNum });
  }, [localDatetime, isLive, lat, lon, onCalculate]);

  return (
    <div className="space-y-6">
      <div className="space-y-1 border-b border-white/10 pb-4">
        <h2 className="text-xl font-serif text-white tracking-widest">天机演算</h2>
        <p className="text-xs text-gray-500 uppercase tracking-wider">Cosmic Calculation</p>
      </div>

      {/* Time Input */}
      <div className="space-y-3">
        <div className="flex justify-between items-center">
           <label className="block text-xs text-gold/70 uppercase tracking-widest">
             观测时间 <span className="text-gray-600 normal-case ml-1">({offsetString})</span>
             {isLive && <span className="ml-2 text-green-400 animate-pulse text-[10px] border border-green-500/30 px-1.5 py-0.5 rounded-full bg-green-500/10">● LIVE</span>}
           </label>
           <button 
             onClick={handleSetNow}
             className={`text-[10px] transition-colors ${isLive ? 'text-gray-600 cursor-default' : 'text-gold/80 hover:text-gold underline decoration-dotted'}`}
             disabled={isLive}
           >
             设为现在
           </button>
        </div>
        <div className="flex items-center gap-2">
          <input
            type="datetime-local"
            value={localDatetime}
            onChange={(e) => {
              setIsLive(false);
              setLocalDatetime(e.target.value);
            }}
            className={`flex-1 bg-black/30 border rounded-lg p-3 text-white font-mono text-sm focus:outline-none focus:ring-1 focus:ring-gold/50 transition-all ${isLive ? 'border-green-500/30' : 'border-white/10 focus:border-gold/50'}`}
          />
        </div>
        {!isLive && (
          <button
            onClick={handleSubmit}
            className="w-full btn-glass-primary py-2 rounded-lg text-xs font-bold uppercase tracking-widest hover:bg-gold/20 transition-colors"
          >
            手动更新时间 (Update Time)
          </button>
        )}
      </div>

      {/* Location Input */}
      <div className="space-y-3 pt-2 border-t border-white/10">
        <div className="flex justify-between items-center">
          <label className="block text-xs text-gold/70 uppercase tracking-widest">观测地点</label>
          <button 
            onClick={handleGeolocation}
            disabled={locationStatus.includes('定位中')}
            className={`text-[10px] flex items-center gap-1 transition-colors px-2 py-1 rounded border ${
              locationStatus.includes('已锁定') 
                ? 'text-green-400 bg-green-950/30 border-green-500/30' 
                : locationStatus.includes('失败')
                ? 'text-red-400 bg-red-950/30 border-red-500/30 hover:border-red-400/50'
                : 'text-cyan-400 hover:text-cyan-300 bg-cyan-950/30 border-cyan-500/30 hover:border-cyan-400/50'
            } disabled:opacity-50`}
          >
            {locationStatus.includes('定位中') ? (
              <svg className="w-3 h-3 animate-spin" fill="none" viewBox="0 0 24 24"><circle className="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" strokeWidth="4"></circle><path className="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"></path></svg>
            ) : (
              <svg className="w-3 h-3" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M17.657 16.657L13.414 20.9a1.998 1.998 0 01-2.827 0l-4.244-4.243a8 8 0 1111.314 0z" /><path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M15 11a3 3 0 11-6 0 3 3 0 016 0z" /></svg>
            )}
            {locationStatus || "获取当前位置"}
          </button>
        </div>
        <div className="grid grid-cols-2 gap-4">
          <div>
            <label className="text-[10px] text-gray-500 mb-1.5 block">纬度 (Lat)</label>
            <input
              type="number"
              step="0.0001"
              value={lat}
              onChange={(e) => setLat(e.target.value)}
              placeholder="39.90"
              className="w-full bg-black/30 border border-white/10 rounded-lg p-2.5 text-white font-mono text-sm focus:border-gold/50 focus:ring-1 focus:ring-gold/50 focus:outline-none transition-all"
            />
          </div>
          <div>
            <label className="text-[10px] text-gray-500 mb-1.5 block">经度 (Lon)</label>
            <input
              type="number"
              step="0.0001"
              value={lon}
              onChange={(e) => setLon(e.target.value)}
              placeholder="116.40"
              className="w-full bg-black/30 border border-white/10 rounded-lg p-2.5 text-white font-mono text-sm focus:border-gold/50 focus:ring-1 focus:ring-gold/50 focus:outline-none transition-all"
            />
          </div>
        </div>
        <button
          onClick={handleSubmit}
          disabled={isLoading}
          className="w-full mt-4 btn-glass-secondary py-3 rounded-lg text-sm font-bold uppercase tracking-widest hover:text-white transition-all disabled:opacity-50 disabled:cursor-not-allowed"
        >
          {isLoading ? '计算中 (Calculating)...' : '开始推演 (Calculate)'}
        </button>
      </div>

      {error && (
        <div className="p-3 bg-red-900/20 border border-red-500/30 rounded text-red-200 text-xs">
          {error}
        </div>
      )}
    </div>
  );
};

export default ControlPanel;
