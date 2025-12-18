/**
 * BaziCard - 人生命盘卡片
 * 整合 BaziForm 和 BaziChartView
 */

import { useState } from 'react';
import BaziForm, { type BaziParams } from './BaziForm';
import BaziChartView, { type BaziResult } from './BaziChartView';
import { Sparkles } from 'lucide-react';

interface BaziCardProps {
  // 从父组件传入的观测时间参数
  observeParams: {
    datetime: string;
    lat: number;
    lon: number;
  };
}

export default function BaziCard({ observeParams }: BaziCardProps) {
  const API_BASE = import.meta.env.VITE_BACKEND_URL || '';
  const [result, setResult] = useState<BaziResult | null>(null);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const handleSubmit = async (params: BaziParams) => {
    setLoading(true);
    setError(null);
    
    try {
      // tzOffsetMinutes: 时区偏移（分钟），东为正 UTC+8=+480, 西为负 UTC-5=-300
      // 注意：与 JS Date.getTimezoneOffset() 符号相反！不要直接使用 getTimezoneOffset() 赋值
      const q = new URLSearchParams({
        datetime: params.datetime,
        timezone: params.timezone,
        tzOffsetMinutes: params.tzOffsetMinutes.toString(),
        lat: params.lat.toString(),
        lon: params.lon.toString(),
        gender: params.gender
      });
      
      const res = await fetch(`${API_BASE}/api/bazi?${q}`);
      
      if (!res.ok) {
        const errText = await res.text();
        throw new Error(errText || '排盘失败');
      }
      
      const data = await res.json();
      setResult(data);
    } catch (err) {
      console.error('Bazi API Error:', err);
      // 网络错误时清空结果，只显示错误信息
      setResult(null);
      if (err instanceof Error) {
        // 检查是否是网络错误
        if (err.message === 'Failed to fetch' || err.name === 'TypeError') {
          setError('服务暂时不可用，请检查网络连接或稍后重试');
        } else {
          setError(err.message);
        }
      } else {
        setError('排盘失败，请稍后重试');
      }
    } finally {
      setLoading(false);
    }
  };

  return (
    <div className="glass-panel rounded-3xl overflow-hidden">
      {/* 标题栏 */}
      <div className="px-6 py-4 border-b border-white/5 flex items-center gap-3">
        <div className="w-8 h-8 rounded-full bg-gradient-to-br from-amber-500/30 to-red-500/30 flex items-center justify-center">
          <Sparkles className="w-4 h-4 text-gold" />
        </div>
        <div>
          <h3 className="text-sm font-bold text-white">人生命盘</h3>
          <p className="text-[10px] text-gray-500 uppercase tracking-widest">Birth Chart · 八字排盘</p>
        </div>
      </div>

      {/* 内容区域 */}
      <div className="p-6 space-y-6">
        {/* 输入表单 */}
        <BaziForm
          observeParams={observeParams}
          onSubmit={handleSubmit}
          isLoading={loading}
        />
        
        {error && (
          <div className="p-3 rounded-xl bg-red-500/10 border border-red-500/20 text-red-300 text-xs">
            {error}
          </div>
        )}
        
        {/* 四柱结果 */}
        <BaziChartView
          data={result}
          isLoading={loading}
        />
      </div>
    </div>
  );
}

