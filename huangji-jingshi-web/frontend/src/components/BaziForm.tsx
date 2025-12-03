/**
 * BaziForm - 八字输入表单
 * 可以跟随"天机演算"的观测时间，或手动输入
 */

import { useState, useEffect } from 'react';
import { Calendar, MapPin, User, Link2, Link2Off, Globe } from 'lucide-react';

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
  const [lat, setLat] = useState(39.9042);
  const [lon, setLon] = useState(116.4074);
  const [gender, setGender] = useState<'male' | 'female' | 'other'>('male');
  const [locationName, setLocationName] = useState('北京');

  // 自动识别时区
  useEffect(() => {
    try {
      const tz = Intl.DateTimeFormat().resolvedOptions().timeZone;
      if (tz) setTimezone(tz);
    } catch {
      // 保持默认
    }
  }, []);

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
      lat: followObserve ? observeParams.lat : lat,
      lon: followObserve ? observeParams.lon : lon,
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
        <label className="flex items-center gap-2 text-xs text-gray-400">
          <MapPin className="w-3.5 h-3.5" />
          出生地点
        </label>
        <div className="grid grid-cols-2 gap-3">
          <div>
            <input
              type="text"
              value={locationName}
              onChange={(e) => setLocationName(e.target.value)}
              placeholder="地名（可选）"
              disabled={followObserve}
              className={`w-full px-3 py-2.5 rounded-xl text-sm transition-all
                ${followObserve 
                  ? 'bg-white/5 text-gray-500 cursor-not-allowed border border-white/5' 
                  : 'bg-black/40 text-white border border-white/10 focus:border-cyan-500/50 focus:outline-none'
                }`}
            />
          </div>
          <div className="flex gap-2">
            <input
              type="number"
              step="0.0001"
              value={followObserve ? observeParams.lat : lat}
              onChange={(e) => setLat(parseFloat(e.target.value))}
              placeholder="纬度"
              disabled={followObserve}
              className={`w-full px-3 py-2.5 rounded-xl text-sm transition-all
                ${followObserve 
                  ? 'bg-white/5 text-gray-500 cursor-not-allowed border border-white/5' 
                  : 'bg-black/40 text-white border border-white/10 focus:border-cyan-500/50 focus:outline-none'
                }`}
            />
            <input
              type="number"
              step="0.0001"
              value={followObserve ? observeParams.lon : lon}
              onChange={(e) => setLon(parseFloat(e.target.value))}
              placeholder="经度"
              disabled={followObserve}
              className={`w-full px-3 py-2.5 rounded-xl text-sm transition-all
                ${followObserve 
                  ? 'bg-white/5 text-gray-500 cursor-not-allowed border border-white/5' 
                  : 'bg-black/40 text-white border border-white/10 focus:border-cyan-500/50 focus:outline-none'
                }`}
            />
          </div>
        </div>
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

