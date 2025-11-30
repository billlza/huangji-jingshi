
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

const ControlPanel: React.FC<ControlPanelProps> = ({ initialParams, onCalculate, isLoading }) => {
  // Local state for inputs
  const getCurrentLocalISO = () => {
    const now = new Date();
    const offset = now.getTimezoneOffset() * 60000;
    return (new Date(now.getTime() - offset)).toISOString().slice(0, 16);
  };
  const initialLive = (() => {
    const paramDate = new Date(initialParams.datetime);
    const now = new Date();
    const diff = Math.abs(paramDate.getTime() - now.getTime());
    return diff <= 60000;
  })();
  const initialDatetime = (() => {
    if (!initialLive) {
      const paramDate = new Date(initialParams.datetime);
      const offset = paramDate.getTimezoneOffset() * 60000;
      return (new Date(paramDate.getTime() - offset)).toISOString().slice(0, 16);
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

  const handleGeolocation = () => {
    setLocationStatus("定位中...");
    if (navigator.geolocation) {
      navigator.geolocation.getCurrentPosition(
        (position) => {
          setLat(position.coords.latitude.toFixed(4));
          setLon(position.coords.longitude.toFixed(4));
          setLocationStatus("GPS已锁定");
          setTimeout(() => setLocationStatus(""), 3000);
        },
        (err) => {
          console.warn("Geolocation error:", err);
          setLocationStatus("定位失败");
          // Fallback logic could go here if needed, but keeping it simple for now
        },
        { timeout: 5000, enableHighAccuracy: true }
      );
    } else {
      setLocationStatus("不支持定位");
    }
  };

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

    // Convert local datetime back to ISO (UTC)
    const date = new Date(localDatetime);
    onCalculate({
      datetime: date.toISOString(),
      lat: latNum,
      lon: lonNum
    });
  };

  // Get timezone string
  // const timezone = Intl.DateTimeFormat().resolvedOptions().timeZone;
  const offsetHours = -(new Date().getTimezoneOffset() / 60);
  const offsetString = `UTC${offsetHours >= 0 ? '+' : ''}${offsetHours}`;

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
    const date = new Date(localDatetime);
    onCalculate({ datetime: date.toISOString(), lat: latNum, lon: lonNum });
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
            className="text-[10px] text-cyan-400 hover:text-cyan-300 flex items-center gap-1 transition-colors bg-cyan-950/30 px-2 py-1 rounded border border-cyan-500/30 hover:border-cyan-400/50"
          >
            <svg className="w-3 h-3" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M17.657 16.657L13.414 20.9a1.998 1.998 0 01-2.827 0l-4.244-4.243a8 8 0 1111.314 0z" /><path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M15 11a3 3 0 11-6 0 3 3 0 016 0z" /></svg>
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
