import React, { useEffect, useState } from 'react';
import type { HuangjiInfo } from '../types';

interface DashboardProps {
  info: HuangjiInfo;
  currentYear: number;
  onJumpToYear?: (year: number) => void;
}

const Dashboard: React.FC<DashboardProps> = ({ info, currentYear, onJumpToYear }) => {
  const [nextLabels, setNextLabels] = useState<{ yun?: string; shi?: string; xun?: string }>();
  const API_BASE = (import.meta as unknown as { env?: Record<string, string> }).env?.VITE_BACKEND_URL || '';
  const SUPABASE_ANON_KEY = (import.meta as unknown as { env?: Record<string, string> }).env?.VITE_SUPABASE_ANON_KEY || '';
  
  useEffect(() => {
    const run = async () => {
      try {
        const nextY = info.yun.end_year + 1;
        const nextS = info.shi.end_year + 1;
        const nextX = info.xun.end_year + 1;
        const fetchOne = async (y: number) => {
          const r = await fetch(`${API_BASE}/functions/v1/timeline?datetime=${y}-06-30T12:00:00Z`, {
            method: 'POST',
            headers: {
              'Authorization': `Bearer ${SUPABASE_ANON_KEY}`,
              'Content-Type': 'application/json'
            }
          });
          const j = await r.json();
          return j as { current: HuangjiInfo };
        };
        const a = await fetchOne(nextY);
        const b = await fetchOne(nextS);
        const c = await fetchOne(nextX);
        setNextLabels({
          yun: `第${a.current.yun.index}运 ${a.current.yun.name}（${a.current.yun.start_year}–${a.current.yun.end_year}）`,
          shi: `第${b.current.shi.index}世 ${b.current.shi.name}（${b.current.shi.start_year}–${b.current.shi.end_year}）`,
          xun: `第${c.current.xun.index}旬 ${c.current.xun.name}（${c.current.xun.start_year}–${c.current.xun.end_year}）`
        });
      } catch {
        setNextLabels(undefined);
      }
    };
    run();
  }, [API_BASE, info.yun.end_year, info.shi.end_year, info.xun.end_year]);
  
  const renderCountdown = (label: string, endYear: number, targetYear?: number, extra?: string) => {
    const yearsLeft = endYear - currentYear;
    return (
      <div className="flex justify-between items-center py-2 border-b border-white/5 last:border-0">
        <span className="text-xs text-gray-500">{label}</span>
        <div className="flex items-center gap-2 text-right">
          <span className={`text-sm font-mono ${yearsLeft < 10 ? 'text-red-400 animate-pulse' : 'text-white'}`}>
            {yearsLeft}
          </span>
          <span className="text-[10px] text-gray-600 ml-1">yrs</span>
          {onJumpToYear && typeof targetYear === 'number' && (
            <button
              onClick={() => onJumpToYear(targetYear)}
              className="text-[10px] text-cyan-400 hover:text-cyan-300 underline decoration-dotted"
            >
              跳转
            </button>
          )}
        </div>
        {extra && (
          <div className="text-[10px] text-gray-500 mt-1 w-full text-right">→ {extra}</div>
        )}
      </div>
    );
  };

  return (
    <div className="bg-black/40 border border-white/5 rounded-lg p-4">
      <h3 className="text-xs text-gold/70 uppercase tracking-widest mb-3">
        临界点 (Critical Points)
      </h3>
      <div className="space-y-1">
        {renderCountdown("Next Yun (运)", info.yun.end_year, info.yun.end_year + 1, nextLabels?.yun)}
        {renderCountdown("Next Shi (世)", info.shi.end_year, info.shi.end_year + 1, nextLabels?.shi)}
        {renderCountdown("Next Xun (旬)", info.xun.end_year, info.xun.end_year + 1, nextLabels?.xun)}
      </div>
    </div>
  );
};

export default Dashboard;
