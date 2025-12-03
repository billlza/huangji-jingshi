/**
 * BaziForm - 八字输入表单
 * 可以跟随"天机演算"的观测时间，或手动输入
 */

import { useState, useEffect } from 'react';
import { Calendar, MapPin, User, Link2, Link2Off, Globe, LocateFixed, Loader2 } from 'lucide-react';

interface BaziFormProps {
  // 从上层传入的观测时间参数
  observeParams: {
    datetime: string;
    lat: number;
    lon: number;
  };
  onSubmit: (params: BaziParams) => void;
  isLoading: boolean;
}

export interface BaziParams {
  datetime: string;  // ISO8601
  timezone: string;  // e.g., "Asia/Shanghai" or "+08:00"
  lat: number;
  lon: number;
  gender: 'male' | 'female' | 'other';
}

// 常用时区列表
const TIMEZONE_OPTIONS = [
  { value: 'Asia/Shanghai', label: '北京时间 (UTC+8)' },
  { value: 'Asia/Tokyo', label: '东京时间 (UTC+9)' },
  { value: 'Asia/Hong_Kong', label: '香港时间 (UTC+8)' },
  { value: 'Asia/Taipei', label: '台北时间 (UTC+8)' },
  { value: 'Asia/Singapore', label: '新加坡时间 (UTC+8)' },
  { value: 'America/New_York', label: '纽约时间 (EST/EDT)' },
  { value: 'America/Los_Angeles', label: '洛杉矶时间 (PST/PDT)' },
  { value: 'Europe/London', label: '伦敦时间 (GMT/BST)' },
  { value: 'Europe/Paris', label: '巴黎时间 (CET/CEST)' },
];

export default function BaziForm({ observeParams, onSubmit, isLoading }: BaziFormProps) {
  // 是否跟随上方的观测时间
  const [followObserve, setFollowObserve] = useState(true);
  
  // 本地状态
  const [localDatetime, setLocalDatetime] = useState('');
  const [timezone, setTimezone] = useState('Asia/Shanghai');
  const [lat, setLat] = useState<number | ''>('');
  const [lon, setLon] = useState<number | ''>('');
  const [gender, setGender] = useState<'male' | 'female' | 'other'>('male');
  const [locationName, setLocationName] = useState('');
  
  // 定位状态
  const [locating, setLocating] = useState(false);
  const [locateError, setLocateError] = useState<string | null>(null);

  // 自动识别时区
  useEffect(() => {
    try {
      const tz = Intl.DateTimeFormat().resolvedOptions().timeZone;
      if (tz) setTimezone(tz);
    } catch {
      // 保持默认
    }
  }, []);

  // 获取定位
  const handleLocate = async () => {
    setLocating(true);
    setLocateError(null);
    
    // 方法1: 尝试 GPS/BDS 定位
    if ('geolocation' in navigator) {
      try {
        const position = await new Promise<GeolocationPosition>((resolve, reject) => {
          navigator.geolocation.getCurrentPosition(resolve, reject, {
            enableHighAccuracy: true,
            timeout: 10000,
            maximumAge: 0
          });
        });
        
        setLat(Number(position.coords.latitude.toFixed(4)));
        setLon(Number(position.coords.longitude.toFixed(4)));
        setLocationName('GPS定位');
        setLocating(false);
        return;
      } catch (gpsError) {
        console.log('GPS定位失败，尝试IP定位...', gpsError);
      }
    }
    
    // 方法2: 尝试 IP 定位
    try {
      const res = await fetch('https://ipapi.co/json/', { 
        signal: AbortSignal.timeout(5000) 
      });
      if (res.ok) {
        const data = await res.json();
        if (data.latitude && data.longitude) {
          setLat(Number(Number(data.latitude).toFixed(4)));
          setLon(Number(Number(data.longitude).toFixed(4)));
          setLocationName(data.city || data.region || 'IP定位');
          setLocating(false);
          return;
        }
      }
    } catch (ipError) {
      console.log('IP定位失败', ipError);
    }
    
    // 都失败了
    setLocateError('定位失败，请手动输入经纬度');
    setLocating(false);
  };

  // 当跟随模式开启时，同步观测时间
  useEffect(() => {
    if (followObserve) {
      setLocalDatetime(observeParams.datetime);
      setLat(observeParams.lat);
      setLon(observeParams.lon);
    }
  }, [followObserve, observeParams]);

  // 格式化日期时间用于 input
  const formatDatetimeLocal = (iso: string) => {
    const d = new Date(iso);
    if (isNaN(d.getTime())) return '';
    const pad = (n: number) => n.toString().padStart(2, '0');
    return `${d.getFullYear()}-${pad(d.getMonth() + 1)}-${pad(d.getDate())}T${pad(d.getHours())}:${pad(d.getMinutes())}`;
  };

  const handleSubmit = (e: React.FormEvent) => {
    e.preventDefault();
    const dt = followObserve ? observeParams.datetime : new Date(localDatetime).toISOString();
    onSubmit({
      datetime: dt,
      timezone,
      lat: followObserve ? observeParams.lat : (lat || 39.9042),  // 默认北京
      lon: followObserve ? observeParams.lon : (lon || 116.4074),
      gender
    });
  };

  return (
    <form onSubmit={handleSubmit} className="space-y-5">
      {/* 跟随观测时间开关 */}
      <div className="flex items-center justify-between p-3 rounded-xl bg-white/5 border border-white/10">
        <div className="flex items-center gap-2">
          {followObserve ? (
            <Link2 className="w-4 h-4 text-cyan-400" />
          ) : (
            <Link2Off className="w-4 h-4 text-gray-500" />
          )}
          <span className="text-xs text-gray-400">跟随观测时间</span>
        </div>
        <button
          type="button"
          onClick={() => setFollowObserve(!followObserve)}
          className={`relative w-10 h-5 rounded-full transition-colors ${
            followObserve ? 'bg-cyan-500/50' : 'bg-white/10'
          }`}
        >
          <div
            className={`absolute top-0.5 w-4 h-4 rounded-full bg-white shadow transition-transform ${
              followObserve ? 'translate-x-5' : 'translate-x-0.5'
            }`}
          />
        </button>
      </div>

      {/* 出生日期时间 */}
      <div className="space-y-2">
        <label className="flex items-center gap-2 text-xs text-gray-400">
          <Calendar className="w-3.5 h-3.5" />
          出生时间
        </label>
        <input
          type="datetime-local"
          value={formatDatetimeLocal(followObserve ? observeParams.datetime : localDatetime)}
          onChange={(e) => setLocalDatetime(e.target.value)}
          disabled={followObserve}
          className={`w-full px-4 py-3 rounded-xl text-sm transition-all
            ${followObserve 
              ? 'bg-white/5 text-gray-500 cursor-not-allowed border border-white/5' 
              : 'bg-black/40 text-white border border-white/10 focus:border-cyan-500/50 focus:outline-none'
            }`}
        />
      </div>

      {/* 时区选择 */}
      <div className="space-y-2">
        <label className="flex items-center gap-2 text-xs text-gray-400">
          <Globe className="w-3.5 h-3.5" />
          时区
        </label>
        <select
          value={timezone}
          onChange={(e) => setTimezone(e.target.value)}
          className="w-full px-4 py-3 rounded-xl bg-black/40 text-white text-sm border border-white/10 focus:border-cyan-500/50 focus:outline-none appearance-none cursor-pointer"
        >
          {TIMEZONE_OPTIONS.map(tz => (
            <option key={tz.value} value={tz.value}>{tz.label}</option>
          ))}
        </select>
      </div>

      {/* 出生地点 */}
      <div className="space-y-2">
        <div className="flex items-center justify-between">
          <label className="flex items-center gap-2 text-xs text-gray-400">
            <MapPin className="w-3.5 h-3.5" />
            出生地点
          </label>
          {!followObserve && (
            <button
              type="button"
              onClick={handleLocate}
              disabled={locating}
              className={`flex items-center gap-1.5 px-2.5 py-1 rounded-lg text-[10px] font-medium transition-all ${
                locating
                  ? 'bg-cyan-500/20 text-cyan-400 cursor-wait'
                  : 'bg-white/5 text-gray-400 hover:bg-cyan-500/20 hover:text-cyan-300 border border-white/10 hover:border-cyan-500/30'
              }`}
            >
              {locating ? (
                <>
                  <Loader2 className="w-3 h-3 animate-spin" />
                  定位中...
                </>
              ) : (
                <>
                  <LocateFixed className="w-3 h-3" />
                  自动定位
                </>
              )}
            </button>
          )}
        </div>
        
        {/* 地名输入 */}
        <input
          type="text"
          value={followObserve ? '' : locationName}
          onChange={(e) => setLocationName(e.target.value)}
          placeholder={followObserve ? '跟随观测地点' : '输入地名（可选）'}
          disabled={followObserve}
          className={`w-full px-3 py-2.5 rounded-xl text-sm transition-all
            ${followObserve 
              ? 'bg-white/5 text-gray-500 cursor-not-allowed border border-white/5' 
              : 'bg-black/40 text-white border border-white/10 focus:border-cyan-500/50 focus:outline-none'
            }`}
        />
        
        {/* 经纬度输入 */}
        <div className="grid grid-cols-2 gap-2">
          <div className="relative">
            <input
              type="number"
              step="0.0001"
              value={followObserve ? observeParams.lat : lat}
              onChange={(e) => setLat(e.target.value ? parseFloat(e.target.value) : '')}
              placeholder="纬度 Lat"
              disabled={followObserve}
              className={`w-full px-3 py-2.5 rounded-xl text-sm transition-all
                ${followObserve 
                  ? 'bg-white/5 text-gray-500 cursor-not-allowed border border-white/5' 
                  : 'bg-black/40 text-white border border-white/10 focus:border-cyan-500/50 focus:outline-none'
                }`}
            />
            <span className="absolute right-3 top-1/2 -translate-y-1/2 text-[9px] text-gray-500">纬度</span>
          </div>
          <div className="relative">
            <input
              type="number"
              step="0.0001"
              value={followObserve ? observeParams.lon : lon}
              onChange={(e) => setLon(e.target.value ? parseFloat(e.target.value) : '')}
              placeholder="经度 Lon"
              disabled={followObserve}
              className={`w-full px-3 py-2.5 rounded-xl text-sm transition-all
                ${followObserve 
                  ? 'bg-white/5 text-gray-500 cursor-not-allowed border border-white/5' 
                  : 'bg-black/40 text-white border border-white/10 focus:border-cyan-500/50 focus:outline-none'
                }`}
            />
            <span className="absolute right-3 top-1/2 -translate-y-1/2 text-[9px] text-gray-500">经度</span>
          </div>
        </div>
        
        {/* 定位错误提示 */}
        {locateError && !followObserve && (
          <div className="text-[10px] text-amber-400 flex items-center gap-1">
            <span>⚠</span> {locateError}
          </div>
        )}
      </div>

      {/* 性别选择 */}
      <div className="space-y-2">
        <label className="flex items-center gap-2 text-xs text-gray-400">
          <User className="w-3.5 h-3.5" />
          性别
        </label>
        <div className="flex gap-2">
          {[
            { value: 'male', label: '男', color: 'cyan' },
            { value: 'female', label: '女', color: 'pink' },
            { value: 'other', label: '其他', color: 'purple' },
          ].map(opt => (
            <button
              key={opt.value}
              type="button"
              onClick={() => setGender(opt.value as typeof gender)}
              className={`flex-1 py-2.5 rounded-xl text-sm font-medium transition-all border ${
                gender === opt.value
                  ? opt.value === 'male'
                    ? 'bg-cyan-500/20 border-cyan-500/50 text-cyan-300'
                    : opt.value === 'female'
                    ? 'bg-pink-500/20 border-pink-500/50 text-pink-300'
                    : 'bg-purple-500/20 border-purple-500/50 text-purple-300'
                  : 'bg-white/5 border-white/10 text-gray-400 hover:bg-white/10'
              }`}
            >
              {opt.label}
            </button>
          ))}
        </div>
      </div>

      {/* 提交按钮 */}
      <button
        type="submit"
        disabled={isLoading}
        className={`w-full py-4 rounded-2xl font-bold text-sm uppercase tracking-wider transition-all
          ${isLoading
            ? 'bg-white/10 text-gray-500 cursor-not-allowed'
            : 'bg-gradient-to-r from-amber-600/80 via-yellow-500/80 to-amber-600/80 text-black hover:shadow-lg hover:shadow-amber-500/20'
          }`}
      >
        {isLoading ? (
          <span className="flex items-center justify-center gap-2">
            <span className="w-4 h-4 border-2 border-gray-500 border-t-transparent rounded-full animate-spin" />
            排盘中...
          </span>
        ) : (
          '排八字命盘'
        )}
      </button>
    </form>
  );
}


