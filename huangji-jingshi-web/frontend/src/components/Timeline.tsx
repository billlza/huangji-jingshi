import React, { useState, useEffect } from 'react';
import type { TimelineData, PeriodInfo, FortuneResponse } from '../types';

interface TimelineProps {
  currentYear: number;
  currentDatetime: string;
  onYearChange: (year: number) => void;
  mapping?: FortuneResponse['mapping_record'];
}

const Timeline: React.FC<TimelineProps> = ({ currentYear, currentDatetime, onYearChange, mapping }) => {
  const API_BASE = import.meta.env.VITE_BACKEND_URL || '';
  const [data, setData] = useState<TimelineData | null>(null);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [showHelp, setShowHelp] = useState(false);
  const [events, setEvents] = useState<Array<{ year: number; title: string; description: string }>>([]);
  const [huiDoc, setHuiDoc] = useState<string | null>(null);

  useEffect(() => {
    const fetchData = async () => {
      setLoading(true);
      try {
        // Assuming the datetime is for the start of the year for simplicity in this view
        const response = await fetch(`${API_BASE}/api/timeline?datetime=${currentYear}-01-01T12:00:00Z`);
        if (!response.ok) throw new Error('Failed to fetch timeline');
        const json = await response.json();
        setData(json);
        setError(null);
      } catch (err) {
        console.error(err);
        setError('Timeline data unavailable');
      } finally {
        setLoading(false);
      }
    };

    fetchData();
  }, [currentYear, API_BASE]);

  useEffect(() => {
    if (!data) return;
    const start = data.current.hui.start_year;
    const end = data.current.hui.end_year;
    const run = async () => {
      try {
        const r = await fetch(`${API_BASE}/api/mapping/get?year=${start}`);
        const j = await r.json();
        const hex = j.record?.nian_hexagram as string | undefined;
        if (hex) setHuiDoc(`${hex}（${start}–${end}）`); else setHuiDoc(null);
      } catch {
        setHuiDoc(null);
      }
    };
    run();
  }, [data, API_BASE]);

  useEffect(() => {
    if (!data) return;
    const rangeFor = (list: PeriodInfo[], activeIndex: number) => {
      const pos = list.findIndex(i => i.index === activeIndex);
      const segs = [
        ...(pos > 0 ? [list[pos - 1]] : []),
        ...(pos >= 0 ? [list[pos]] : []),
        ...(pos >= 0 && pos < list.length - 1 ? [list[pos + 1]] : []),
      ];
      return {
        start: Math.min(...segs.map(s => s.start_year)),
        end: Math.max(...segs.map(s => s.end_year)),
      };
    };
    const r1 = rangeFor(data.hui_list, data.current.hui.index);
    const r2 = rangeFor(data.yun_list, data.current.yun.index);
    const r3 = rangeFor(data.shi_list, data.current.shi.index);
    const r4 = rangeFor(data.xun_list, data.current.xun.index);
    const start = Math.min(r1.start, r2.start, r3.start, r4.start);
    const end = Math.max(r1.end, r2.end, r3.end, r4.end);
    const fetchEvents = async () => {
      try {
        const resp = await fetch(`${API_BASE}/api/history?start=${start}&end=${end}`);
        if (!resp.ok) throw new Error('Failed');
        const json = await resp.json();
        setEvents(json);
      } catch {
        setEvents([]);
      }
    };
    fetchEvents();
  }, [data, API_BASE]);

  if (loading && !data) return <div className="h-32 bg-white/5 animate-pulse rounded-lg" />;
  if (error) return <div className="text-red-400 text-xs p-4 border border-red-900 rounded-lg">{error}</div>;
  if (!data) return null;

  const dtObj = new Date(currentDatetime);
  const tzOffset = -dtObj.getTimezoneOffset() / 60;
  const tzSign = tzOffset >= 0 ? '+' : '-';
  const tzLabel = `UTC${tzSign}${String(Math.abs(Math.floor(tzOffset))).padStart(2, '0')}`;
  const localStr = `${dtObj.getFullYear()}-${String(dtObj.getMonth() + 1).padStart(2, '0')}-${String(dtObj.getDate()).padStart(2, '0')} ${String(dtObj.getHours()).padStart(2, '0')}:${String(dtObj.getMinutes()).padStart(2, '0')}`;

  const renderEvents = (period: PeriodInfo) => {
    const eventsInPeriod = events.filter(e => e.year >= period.start_year && e.year <= period.end_year);
    if (eventsInPeriod.length === 0) return null;
    
    return (
        <div className="absolute bottom-0 left-0 right-0 flex justify-center space-x-1 pb-1">
            {eventsInPeriod.map(e => (
                <div key={e.title} className="w-1 h-1 bg-red-500 rounded-full" title={`${e.year}: ${e.title}`} />
            ))}
        </div>
    );
  };

  const renderRow = (title: string, items: PeriodInfo[], activeIndex: number, colorClass: string) => {
    let centerPos = items.findIndex((i) => i.index === activeIndex);
    if (centerPos < 0 && items.length > 0) centerPos = Math.min(items.length - 1, Math.floor(items.length / 2));
    const displayItems: PeriodInfo[] = [];
    if (centerPos > 0) displayItems.push(items[centerPos - 1]);
    if (centerPos >= 0 && centerPos < items.length) displayItems.push(items[centerPos]);
    if (centerPos >= 0 && centerPos < items.length - 1) displayItems.push(items[centerPos + 1]);
    let activeItem = items.find(i => i.index === activeIndex);
    if (!activeItem && centerPos >= 0 && items.length > 0) activeItem = items[centerPos];
    const typeLabel = title.includes('Era') ? '会' : title.includes('Cycle') ? '运' : title.includes('Gen') ? '世' : '旬';
    const spanLabel = title.includes('Era') ? '10800年' : title.includes('Cycle') ? '360年' : title.includes('Gen') ? '30年' : '10年';
    const labelForItem = (item: PeriodInfo) => {
      if (typeLabel === '会') return `${item.name}会`;
      if (typeLabel === '运') return `第${item.index}运`;
      if (typeLabel === '世') return `第${item.index}世`;
      return `第${item.index}旬`;
    };
    return (
      <div className="mb-6">
        <div className="flex items-center justify-between mb-2">
            <div className="relative group">
              <h4 className="text-[10px] text-gray-400 uppercase tracking-widest w-24 shrink-0">{title}</h4>
              <div className="absolute right-full top-1/2 -translate-y-1/2 mr-2 w-64 bg-black/90 border border-white/20 rounded p-2 hidden group-hover:block z-20 pointer-events-none">
                <div className="text-[10px] text-gray-400 mb-1 text-center">{typeLabel} · {spanLabel}</div>
                {activeItem && (
                  <div className="text-[10px] text-gray-200 text-center">
                    当前：{activeItem.name}
                  </div>
                )}
                {activeItem && (
                  <div className="text-[10px] text-gray-500 text-center">公元 {activeItem.start_year} – {activeItem.end_year}</div>
                )}
              </div>
            </div>
            <div className="h-[1px] bg-white/10 flex-grow ml-4"></div>
        </div>
        <div className="text-[10px] text-gray-500 mb-2">
          {activeItem ? (
            <>
              {typeLabel}：{activeItem.name} · 公元 {activeItem.start_year} – {activeItem.end_year}
            </>
          ) : (
            <>
              {typeLabel} · {spanLabel}
            </>
          )}
        </div>
        
        <div className="flex space-x-1 overflow-x-auto pb-2 scrollbar-thin scrollbar-thumb-white/20 scrollbar-track-transparent">
          {displayItems.map((item) => {
            const isActive = item.index === activeIndex;
            const isPast = item.end_year < currentYear;
            
            return (
              <button
                key={`${title}-${item.index}`}
                onClick={() => onYearChange(item.start_year)}
                className={`
                  flex-shrink-0 relative group transition-all duration-300
                  ${isActive ? 'w-32' : 'w-20 hover:w-24'}
                  h-16 rounded-sm border border-white/5 overflow-hidden
                  ${isActive ? 'bg-white/10 border-white/20' : 'bg-white/5 hover:bg-white/10'}
                `}
              >
                {/* Background Fill for Active/Past */}
                <div className={`absolute inset-0 opacity-20 ${isActive ? colorClass : isPast ? 'bg-gray-800' : ''}`} />
                
                {/* Active Indicator Line */}
                {isActive && (
                    <div className={`absolute top-0 left-0 right-0 h-0.5 ${colorClass} shadow-[0_0_8px_currentColor]`} />
                )}

                <div className="absolute inset-0 p-2 flex flex-col justify-between">
                  <span className={`text-[10px] font-mono truncate ${isActive ? 'text-white' : 'text-gray-500 group-hover:text-gray-300'}`}>
                    {labelForItem(item)}
                  </span>
                  
                  <span className="text-[9px] text-gray-600 font-mono absolute bottom-1 right-2 opacity-50 group-hover:opacity-100">
                      {item.index}
                  </span>
                  
                  {isActive && (
                      <div className="text-[9px] text-gray-400 font-mono mt-1">
                          {item.start_year}
                      </div>
                  )}
                  
                  {/* Event Markers */}
                  {renderEvents(item)}
                </div>
                
                {/* Hover Tooltip for Structure */}
                <div className="absolute bottom-full left-1/2 -translate-x-1/2 mb-2 w-56 bg-black/90 border border-white/20 rounded p-2 hidden group-hover:block z-20 pointer-events-none">
                    <div className="text-[10px] text-gray-400 mb-1 text-center">公元 {item.start_year} – {item.end_year} · {labelForItem(item)}</div>
                    {typeLabel === '运' && (
                      <div className="text-[10px] text-gray-200 text-center">
                        {mapping?.yun_raw ? mapping.yun_raw : `${item.name}（${item.start_year}-${item.end_year}）`}
                      </div>
                    )}
                    {typeLabel === '会' && (
                      <div className="text-[10px] text-gray-200 text-center">
                        {(mapping?.hui_raw || `${item.name}会`) + (item.index === activeIndex && huiDoc ? ` · ${huiDoc}` : '')}
                      </div>
                    )}
                    {typeLabel === '旬' && (
                      <div className="text-[10px] text-gray-200 text-center">
                        {mapping?.xun_raw || `第${item.index}旬`}
                      </div>
                    )}
                </div>
              </button>
            );
          })}
        </div>
      </div>
    );
  };

  return (
    <div id="timeline-section" className="bg-black/40 backdrop-blur-sm border border-white/10 rounded-xl p-6 w-full relative">
      <div className="flex items-center justify-between mb-6">
          <h3 className="text-sm font-serif text-gold tracking-widest flex items-center">
              <span className="w-2 h-2 bg-gold rounded-full mr-2 animate-pulse"></span>
              皇极经世 · 时间轴
              <button 
                onClick={() => setShowHelp(!showHelp)}
                className="ml-3 w-5 h-5 rounded-full border border-gold/50 text-gold/70 flex items-center justify-center text-xs hover:bg-gold hover:text-black transition-all duration-300 focus:outline-none focus:ring-2 focus:ring-gold/30"
                title="查看说明"
              >
                ?
              </button>
          </h3>
          <div className="flex items-center space-x-4">
            <button 
                onClick={() => onYearChange(new Date().getFullYear())}
                className="text-[10px] text-gold/70 hover:text-gold border border-gold/30 px-2 py-0.5 rounded transition-colors"
            >
                回到现在
            </button>
            <div className="text-[10px] text-gray-500 font-mono">
                CENTER: {data.current.year_gua} ({currentYear}) · {localStr} {tzLabel}
            </div>
          </div>
      </div>

      {/* Help Overlay - Modal Style */}
      {showHelp && (
        <div className="fixed inset-0 z-[9999] flex items-center justify-center p-4 bg-black/60 backdrop-blur-sm animate-in fade-in duration-200" onClick={() => setShowHelp(false)}>
          <div 
            className="bg-white w-full max-w-md rounded-xl shadow-2xl p-6 relative animate-in zoom-in-95 duration-200" 
            onClick={e => e.stopPropagation()}
            style={{ backgroundColor: '#ffffff', color: '#1f2937' }} // Inline style fallback
          >
             <button 
                onClick={() => setShowHelp(false)} 
                className="absolute top-4 right-4 w-8 h-8 flex items-center justify-center rounded-full bg-gray-100 hover:bg-gray-200 text-gray-500 hover:text-gray-700 transition-colors"
             >
               ✕
             </button>
             
             <h4 className="text-xl font-serif font-bold text-gray-900 mb-1">时间层级说明</h4>
             <p className="text-xs text-gray-500 mb-5">皇极经世 · 元会运世</p>

             <ul className="space-y-4">
               <li className="flex items-start p-3 rounded-lg hover:bg-gray-50 transition-colors">
                 <div className="w-3 h-3 mt-1.5 rounded-full bg-cyan-500 mr-4 shrink-0 shadow-sm"></div>
                 <div>
                   <strong className="text-gray-900 block text-sm font-bold">会 (Era) - 10,800年</strong>
                   <p className="text-xs text-gray-600 mt-1 leading-relaxed">
                     宇宙的大月。共12会（子至亥）。<br/>
                     我们处于<span className="text-cyan-600 font-medium">“午会”</span>，正如正午时分，是人类文明最极盛、最光明的黄金时期。
                   </p>
                 </div>
               </li>
               <li className="flex items-start p-3 rounded-lg hover:bg-gray-50 transition-colors">
                 <div className="w-3 h-3 mt-1.5 rounded-full bg-emerald-500 mr-4 shrink-0 shadow-sm"></div>
                 <div>
                   <strong className="text-gray-900 block text-sm font-bold">运 (Cycle) - 360年</strong>
                   <p className="text-xs text-gray-600 mt-1 leading-relaxed">
                     宇宙的大日。一运360年，对应历史上大的朝代兴衰周期。<br/>
                     我们处于<span className="text-emerald-600 font-medium">“姤运”</span>（1744-2103），阴长阳消之始。
                   </p>
                 </div>
               </li>
               <li className="flex items-start p-3 rounded-lg hover:bg-gray-50 transition-colors">
                 <div className="w-3 h-3 mt-1.5 rounded-full bg-amber-500 mr-4 shrink-0 shadow-sm"></div>
                 <div>
                   <strong className="text-gray-900 block text-sm font-bold">世 (Generation) - 30年</strong>
                   <p className="text-xs text-gray-600 mt-1 leading-relaxed">
                     古语“三十年为一世”，代表一代人的时间跨度。<br/>
                     我们处于<span className="text-amber-600 font-medium">“鼎世”</span>（2014-2043），革故鼎新，去旧生新。
                   </p>
                 </div>
               </li>
               <li className="flex items-start p-3 rounded-lg hover:bg-gray-50 transition-colors">
                 <div className="w-3 h-3 mt-1.5 rounded-full bg-purple-500 mr-4 shrink-0 shadow-sm"></div>
                 <div>
                   <strong className="text-gray-900 block text-sm font-bold">旬 (Decade) - 10年</strong>
                   <p className="text-xs text-gray-600 mt-1 leading-relaxed">
                     最小的时间刻度，用于细微的流年推演。
                   </p>
                 </div>
               </li>
             </ul>
             
             <div className="mt-6 pt-4 border-t border-gray-100 text-[10px] text-gray-400 text-center flex items-center justify-center gap-2">
               <span className="w-1.5 h-1.5 bg-gray-300 rounded-full animate-pulse"></span>
               提示：点击时间轴上的色块，可跳转至对应历史时期查看卦象与事件
             </div>
          </div>
        </div>
      )}

      <div className="space-y-2">
        {renderRow("会 (Era)", data.hui_list, data.current.hui.index, "bg-cyan-500")}
        {renderRow("运 (Cycle)", data.yun_list, data.current.yun.index, "bg-emerald-500")}
        {renderRow("世 (Generation)", data.shi_list, data.current.shi.index, "bg-amber-500")}
        {renderRow("旬 (Decade)", data.xun_list, data.current.xun.index, "bg-purple-500")}
      </div>
    </div>
  );
};

export default Timeline;
