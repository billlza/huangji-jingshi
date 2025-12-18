/**
 * BaziForm - 八字输入表单
 * 可以跟随"天机演算"的观测时间，或手动输入
 */

import { useState, useEffect } from 'react';
import { Calendar, MapPin, User, Link2, Link2Off, Globe, LocateFixed, Loader2 } from 'lucide-react';
import { reverseGeocode, getIPLocation, geocode } from '../utils/geolocation';
import { convertLocalToUTC, getTimezoneOffsetMinutes } from '../utils/timezoneConvert';

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
  datetime: string;  // ISO8601 UTC 时间
  timezone: string;  // e.g., "Asia/Shanghai" or "+08:00"
  // tzOffsetMinutes: 时区偏移（分钟），东为正 UTC+8=+480, 西为负 UTC-5=-300
  // 注意：与 JS Date.getTimezoneOffset() 符号相反！不要直接使用 getTimezoneOffset() 赋值
  tzOffsetMinutes: number;
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
  const [geocoding, setGeocoding] = useState(false);

  // 注：不再自动检测设备时区，默认使用 Asia/Shanghai
  // 用户可通过下拉菜单手动选择其他时区

  // 获取定位
  const handleLocate = async () => {
    setLocating(true);
    setLocateError(null);
    
    let latitude: number | null = null;
    let longitude: number | null = null;
    let source = '';
    
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
        
        latitude = Number(position.coords.latitude.toFixed(4));
        longitude = Number(position.coords.longitude.toFixed(4));
        source = 'GPS';
        
        console.log('✅ GPS定位成功:', latitude, longitude);
      } catch (gpsError) {
        console.log('❌ GPS定位失败，尝试IP定位...', gpsError);
      }
    }
    
    // 方法2: 如果GPS失败，尝试 IP 定位（智能路由）
    if (latitude === null || longitude === null) {
      try {
        const location = await getIPLocation();
        
        if (location) {
          latitude = Number(location.latitude.toFixed(4));
          longitude = Number(location.longitude.toFixed(4));
          source = 'IP';
          
          // IP定位直接返回城市名
          setLat(latitude);
          setLon(longitude);
          setLocationName(location.city || location.region || '未知地点');
          setLocating(false);
          return;
        }
      } catch (ipError) {
        console.log('❌ IP定位失败', ipError);
      }
    }
    
    // 如果获取到了坐标（GPS或IP）
    if (latitude !== null && longitude !== null) {
      setLat(latitude);
      setLon(longitude);
      
      // 进行逆地理编码获取地名（智能路由）
      const locationName = await reverseGeocode(latitude, longitude);
      if (locationName) {
        setLocationName(locationName);
      } else {
        // 如果逆地理编码失败，显示来源标识
        setLocationName(source === 'GPS' ? 'GPS定位' : 'IP定位');
      }
      
      setLocating(false);
      return;
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
    
    // 获取时区偏移（分钟）
    // tzOffsetMinutes: 东为正 UTC+8=+480, 西为负 UTC-5=-300
    // 注意：不要使用 getTimezoneOffset() 直接赋值（符号相反）
    const tzOffsetMinutes = getTimezoneOffsetMinutes(timezone);
    
    let dt: string;
    if (followObserve) {
      dt = observeParams.datetime;
    } else {
      // 关键修复：根据用户选择的时区偏移，将本地时间字符串正确转换为UTC
      // 显式使用 tzOffsetMinutes，不依赖浏览器时区
      dt = convertLocalToUTC(localDatetime, tzOffsetMinutes);
    }
    
    onSubmit({
      datetime: dt,
      timezone,
      tzOffsetMinutes,
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
        <div className="relative">
          <input
            type="text"
            value={followObserve ? '' : locationName}
            onChange={(e) => setLocationName(e.target.value)}
            onBlur={async (e) => {
              // 当输入框失去焦点且地址不为空时，自动进行地理编码
              // 检查是否点击的是模态框或其他元素，避免误触发
              const relatedTarget = (e.nativeEvent as FocusEvent).relatedTarget;
              if (relatedTarget && (relatedTarget as HTMLElement).closest('[role="dialog"], .modal, [data-modal]')) {
                return; // 如果焦点转移到模态框，不触发地理编码
              }
              
              if (!followObserve && locationName.trim() && locationName.trim().length > 2) {
                setGeocoding(true);
                setLocateError(null);
                try {
                  const result = await geocode(locationName.trim());
                  if (result) {
                    // 确保经纬度正确更新
                    setLat(Number(result.latitude.toFixed(6)));
                    setLon(Number(result.longitude.toFixed(6)));
                    setLocationName(result.address); // 使用标准化的地址
                    console.log('✅ 地理编码成功:', result);
                    console.log('✅ 更新经纬度:', result.latitude, result.longitude);
                  } else {
                    setLocateError('无法找到该地址，请检查地址是否正确或手动输入经纬度');
                  }
                } catch (error) {
                  console.error('地理编码失败:', error);
                  setLocateError('地址解析失败，请手动输入经纬度');
                } finally {
                  setGeocoding(false);
                }
              }
            }}
            onKeyDown={(e) => {
              // 按回车键时也进行地理编码
              if (e.key === 'Enter' && !followObserve && locationName.trim() && locationName.trim().length > 2) {
                e.preventDefault();
                (e.target as HTMLInputElement).blur();
              }
            }}
            placeholder={followObserve ? '跟随观测地点' : '输入地名，如：浙江省杭州市西湖区'}
            disabled={followObserve}
            className={`w-full px-3 py-2.5 rounded-xl text-sm transition-all pr-10
              ${followObserve 
                ? 'bg-white/5 text-gray-500 cursor-not-allowed border border-white/5' 
                : 'bg-black/40 text-white border border-white/10 focus:border-cyan-500/50 focus:outline-none'
              }`}
          />
          {geocoding && !followObserve && (
            <div className="absolute right-3 top-1/2 -translate-y-1/2">
              <Loader2 className="w-4 h-4 animate-spin text-cyan-400" />
            </div>
          )}
        </div>
        
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


