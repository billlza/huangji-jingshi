import React, { useState, useEffect } from 'react';
import type { TimelineData, PeriodInfo, FortuneResponse } from '../types';

interface TimelineProps {
  currentYear: number;
  currentDatetime: string;
  onYearChange: (year: number) => void;
  mapping?: FortuneResponse['mapping_record'];
}

// 静态的皇极经世时间轴数据
const HUANGJI_TIMELINE = {
  yuan: { name: '元', current: '中天', startYear: 960, endYear: 12960 },
  hui: { name: '会', current: '午会', startYear: -2216, endYear: 8583 },
  yun: { name: '运', current: '第12运 姤', startYear: 1744, endYear: 2103 },
  shi: { name: '世', current: '第10世 鼎', startYear: 2014, endYear: 2043 },
  xun: { name: '旬', current: '第2旬', startYear: 2024, endYear: 2033 }
};

const Timeline: React.FC<TimelineProps> = ({ currentYear, currentDatetime, onYearChange, mapping }) => {
  const API_BASE = import.meta.env.VITE_BACKEND_URL || '';
  const SUPABASE_ANON_KEY = import.meta.env.VITE_SUPABASE_ANON_KEY || '';
  const [data, setData] = useState<TimelineData | null>(null);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [showHelp, setShowHelp] = useState(false);

  // 尝试获取timeline数据，但不影响显示
  useEffect(() => {
    const fetchData = async () => {
      setLoading(true);
      try {
        const response = await fetch(`${API_BASE}/functions/v1/timeline`, {
          method: 'POST',
          headers: {
            'Authorization': `Bearer ${SUPABASE_ANON_KEY}`,
            'Content-Type': 'application/json'
          },
          body: JSON.stringify({
            datetime: `${currentYear}-01-01T12:00:00Z`,
            birthDate: `${currentYear}-01-01T12:00:00Z`
          })
        });
        if (response.ok) {
          const json = await response.json();
          setData(json);
        } else {
          console.log('Timeline API not available, using static data');
        }
        setError(null);
      } catch (err) {
        console.log('Timeline API failed, using static data:', err);
      } finally {
        setLoading(false);
      }
    };

    fetchData();
  }, [currentYear, API_BASE]);

  const dtObj = new Date(currentDatetime);
  const tzOffset = -dtObj.getTimezoneOffset() / 60;
  const tzSign = tzOffset >= 0 ? '+' : '-';
  const tzLabel = `UTC${tzSign}${String(Math.abs(Math.floor(tzOffset))).padStart(2, '0')}`;
  const localStr = `${dtObj.getFullYear()}-${String(dtObj.getMonth() + 1).padStart(2, '0')}-${String(dtObj.getDate()).padStart(2, '0')} ${String(dtObj.getHours()).padStart(2, '0')}:${String(dtObj.getMinutes()).padStart(2, '0')}`;

  // 使用静态数据渲染皇极经世时间轴
  const renderTimelineRow = (title: string, info: any, type: string, colorClass: string) => {
    return (
      <div className="flex items-center space-x-3">
        <div className={`w-16 text-xs text-gray-300 ${colorClass} font-semibold`}>
          {title}
        </div>
        <div className="flex-1 flex items-center space-x-2">
          <span className="text-sm text-gray-400">{info.name}:</span>
          <span className={`text-sm font-medium px-3 py-1 rounded-full ${colorClass} bg-opacity-20`}>
            {info.current}
          </span>
          <span className="text-xs text-gray-500">
            {info.startYear} - {info.endYear}
          </span>
        </div>
      </div>
    );
  };

  return (
    <div className="w-full p-6">
      {/* 标题和帮助 */}
      <div className="flex items-center justify-between mb-6">
        <div className="flex items-center gap-2">
          <h2 className="text-lg font-bold text-transparent bg-clip-text bg-gradient-to-r from-gold to-yellow-200">
            皇极经世·时间轴
          </h2>
          <button
            onClick={() => setShowHelp(!showHelp)}
            className="text-gray-400 hover:text-gold transition-colors"
            title="查看帮助"
          >
            <svg className="w-4 h-4" fill="currentColor" viewBox="0 0 20 20">
              <path fillRule="evenodd" d="M18 10a8 8 0 11-16 0 8 8 0 0116 0zm-8-3a1 1 0 00-.867.5 1 1 0 11-1.731-1A3 3 0 0113 8a3.001 3.001 0 01-2 2.83V11a1 1 0 11-2 0v-1a1 1 0 011-1 1 1 0 100-2zm0 8a1 1 0 100-2 1 1 0 000 2z" clipRule="evenodd" />
            </svg>
          </button>
        </div>
        <div className="text-xs text-gray-400">
          CENTER: {localStr} {tzLabel}
        </div>
      </div>

      {/* 帮助信息 */}
      {showHelp && (
        <div className="mb-6 p-4 bg-gray-900/50 rounded-lg border border-gray-700">
          <p className="text-xs text-gray-300 leading-relaxed">
            皇极经世是宋代邵雍所创的宇宙时间体系，将时间划分为：
            元（12960年）→ 会（1080年）→ 运（360年）→ 世（30年）→ 旬（10年）的层级结构。
            当前显示的是基于2025年的皇极经世时间定位。
          </p>
        </div>
      )}

      {/* 时间轴显示 */}
      <div className="space-y-4">
        {renderTimelineRow('元', HUANGJI_TIMELINE.yuan, 'yuan', 'text-blue-400 border-blue-400')}
        {renderTimelineRow('会', HUANGJI_TIMELINE.hui, 'hui', 'text-purple-400 border-purple-400')}
        {renderTimelineRow('运', HUANGJI_TIMELINE.yun, 'yun', 'text-cyan-400 border-cyan-400')}
        {renderTimelineRow('世', HUANGJI_TIMELINE.shi, 'shi', 'text-yellow-400 border-yellow-400')}
        {renderTimelineRow('旬', HUANGJI_TIMELINE.xun, 'xun', 'text-green-400 border-green-400')}
      </div>

      {/* 加载状态 */}
      {loading && (
        <div className="mt-4 text-xs text-gray-400">
          正在同步时间轴数据...
        </div>
      )}
    </div>
  );
};

export default Timeline;