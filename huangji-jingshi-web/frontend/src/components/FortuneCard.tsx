import React, { useEffect, useMemo, useState } from 'react';
import type { FortuneResponse } from '../types';
import Hexagram from '../Hexagram';
import Dashboard from './Dashboard';

interface FortuneCardProps {
  data: FortuneResponse;
  currentYear?: number;
  onJumpToYear?: (year: number) => void;
}

const FortuneCard: React.FC<FortuneCardProps> = ({ data, currentYear, onJumpToYear }) => {
  const [isExpanded, setIsExpanded] = useState(false);
  const [detailsOpen, setDetailsOpen] = useState(false);
  const [mirrorItems, setMirrorItems] = useState<Array<{ year: number; title: string; dynasty?: string; person?: string }>>([]);
  const API_BASE = (import.meta as unknown as { env?: Record<string, string> }).env?.VITE_BACKEND_URL || '';
  
  // If currentYear is not passed, try to parse from datetime ISO if available in context, 
  // or fallback to system year (which might be wrong for history view, but better than nothing).
  // Ideally App should pass it.
  const displayYear = currentYear || new Date().getFullYear();

  const KEYWORDS: Record<string, string> = {
    "乾": "大创、元亨、领导、开端",
    "坤": "承载、顺势、包容、母性",
    "革": "变革、去旧生新、更新换代",
    "鼎": "鼎新、完成、器成、制度化",
    "姤": "遇合、突发、相逢、转折",
    "离": "光明、文明、依附、火性",
    "坎": "险难、考验、沉潜、水性",
  };
  const SUMMARY: Record<string, string> = {
    "革": "革，己日乃孚。改旧成新，适时而动，顺天应人。",
    "鼎": "鼎，元吉。器成而用，烹饪以济，革故鼎新。",
    "乾": "乾，元亨利贞。天行健，君子以自强不息。",
    "坤": "坤，元亨利牝马之贞。地势坤，君子以厚德载物。",
  };

  const yunRange = useMemo(() => {
    const start = data.period_info?.yun.start_year;
    const end = data.period_info?.yun.end_year;
    return start !== undefined && end !== undefined ? { start, end } : null;
  }, [data.period_info]);

  useEffect(() => {
    const run = async () => {
      try {
        if (!yunRange) return;
        const resp = await fetch(`${API_BASE}/api/history/related?year=${displayYear}&mode=yun&limit=3`);
        if (!resp.ok) return;
        const list: Array<{ year: number; title: string; dynasty?: string; person?: string }> = await resp.json();
        setMirrorItems(list);
      } catch {
        setMirrorItems([]);
      }
    };
    run();
  }, [API_BASE, displayYear, yunRange]);

  return (
    <div className="space-y-6">
      {/* Top Section: Dashboard Only */}
      {data.period_info && (
        <div className="grid grid-cols-1 gap-6">
           <div>
             <Dashboard info={data.period_info} currentYear={displayYear} onJumpToYear={onJumpToYear} />
           </div>
        </div>
      )}

      {/* Main Card */}
      <div className="glass-panel rounded-3xl overflow-hidden flex flex-col">
        {/* Header: Main Hexagram */}
        <div className="p-8 border-b border-white/10 flex items-center justify-between bg-white/5">
          <div>
            <div className="text-xs text-gold/60 uppercase tracking-[0.3em] mb-3">值年卦象 (Annual Hexagram)</div>
            <h2 className="text-4xl font-serif text-white tracking-widest drop-shadow-lg mb-2">{data.hexagram_major}</h2>
            <div className="text-sm text-gray-300 font-mono mb-3">
              {data.lunar?.ganzhi_year}年 · {data.lunar?.zodiac}
            </div>
            <div className="text-sm text-gold/90 bg-gold/10 px-3 py-1 rounded-full inline-block border border-gold/20">
              关键词：{KEYWORDS[data.hexagram_major] || "无关键词数据"}
            </div>
          </div>
          <div className="transform scale-90 origin-right drop-shadow-[0_0_15px_rgba(212,175,55,0.3)]">
            <Hexagram name={data.hexagram_major} code={data.hexagram_code} />
          </div>
        </div>

        {/* Key Metrics Chips (Legacy view, maybe keep for quick reference) */}
        <div className="p-4 flex flex-wrap gap-3 bg-black/20 border-b border-white/5 backdrop-blur-sm">
          <Chip label="元" value={data.yuan} color="text-white" />
          <Chip label="会" value={data.hui} color="text-cyan-300" />
          <Chip label="运" value={data.yun} color="text-gold" />
          <Chip label="世" value={data.shi} color="text-gold" />
          <Chip label="旬" value={data.xun} color="text-purple-300" />
        </div>

        {/* Brief Explanation */}
        <div className="p-8 space-y-6">
          <div className="bg-white/5 p-6 rounded-2xl border border-white/5 hover:border-gold/20 transition-colors">
             <p className="text-base text-gray-300 leading-relaxed font-light">
               {data.note.length > 100 && !isExpanded ? `${data.note.slice(0, 100)}...` : data.note}
             </p>
             <button 
               onClick={() => setIsExpanded(!isExpanded)}
               className="text-xs text-gold/70 hover:text-gold mt-4 flex items-center gap-1"
             >
               {isExpanded ? "收起详情" : "展开详情"}
             </button>
          </div>

          {/* Historical Mirror */}
          <div className="bg-black/30 p-6 rounded-2xl border border-white/5">
            <div className="text-xs text-gold/70 uppercase tracking-widest mb-3">历史映照 · HISTORICAL MIRROR</div>
            <div className="space-y-2 text-sm">
              {mirrorItems.length === 0 && (
                <div className="text-gray-500 text-xs">暂无典型记录</div>
              )}
              {mirrorItems.map(m => (
                <div key={`${m.year}-${m.title}`} className="border-b border-white/5 pb-2 last:border-0">
                  <div className="flex items-center justify-between">
                    <div className="text-white">● 公元 {m.year}  {m.title}</div>
                    {onJumpToYear && (
                      <button onClick={() => onJumpToYear(m.year)} className="text-[10px] text-cyan-400 hover:text-cyan-300 underline decoration-dotted">跳转</button>
                    )}
                  </div>
                  {(m.dynasty || m.person) && (
                    <div className="text-[12px] text-gray-400 mt-1">{m.dynasty || ''} {m.person || ''}</div>
                  )}
                </div>
              ))}
            </div>
          </div>

          {/* Expanded Details */}
          {isExpanded && (
            <div className="mt-4 space-y-4 animate-fadeIn">
               {data.lunar && (
                 <div className="grid grid-cols-2 gap-4 text-xs">
                    <div className="bg-black/40 p-3 rounded border border-white/5">
                      <span className="block text-gray-500 mb-1">节气 (Solar Term)</span>
                      <span className="text-white font-serif">{data.lunar.solar_term || "N/A"}</span>
                    </div>
                    <div className="bg-black/40 p-3 rounded border border-white/5">
                      <span className="block text-gray-500 mb-1">建除 (Officer)</span>
                      <span className="text-white font-serif">{data.lunar.twelve_officer}</span>
                    </div>
                    <div className="bg-black/40 p-3 rounded border border-white/5 col-span-2">
                       <span className="block text-gray-500 mb-1">宜 (Aus)</span>
                       <span className="text-green-400">{data.lunar.yi.join(' ')}</span>
                    </div>
                    <div className="bg-black/40 p-3 rounded border border-white/5 col-span-2">
                       <span className="block text-gray-500 mb-1">忌 (Ominous)</span>
                       <span className="text-red-400">{data.lunar.ji.join(' ')}</span>
                    </div>
                 </div>
               )}
               <div className="bg-black/40 p-4 rounded border border-white/5 text-xs">
                 <div className="text-gray-500 mb-1">卦辞摘要</div>
                 <div className="text-gray-200">{SUMMARY[data.hexagram_major] || "暂无摘要"}</div>
               </div>
              {data.period_info && (
                <div className="grid grid-cols-3 gap-3 text-xs">
                   <div className="bg-black/40 p-3 rounded border border-white/5">
                     <div className="text-gray-500 mb-1">运</div>
                     <div className="text-white font-serif">{data.period_info.yun.name}</div>
                   </div>
                   <div className="bg-black/40 p-3 rounded border border-white/5">
                     <div className="text-gray-500 mb-1">世</div>
                     <div className="text-white font-serif">{data.period_info.shi.name}</div>
                   </div>
                   <div className="bg-black/40 p-3 rounded border border-white/5">
                     <div className="text-gray-500 mb-1">旬</div>
                     <div className="text-white font-serif">{data.period_info.xun.name}</div>
                   </div>
                </div>
              )}
              <div className="pt-2">
                <button 
                  onClick={() => setDetailsOpen(true)}
                  className="text-[12px] text-gold/80 hover:text-gold underline decoration-dotted"
                >
                  展开推演明细 / Open detailed derivation
                </button>
              </div>
            </div>
          )}
        </div>
      </div>

      {/* Details Drawer */}
      {detailsOpen && (
        <div className="fixed inset-0 z-50">
          <div className="absolute inset-0 bg-black/60" onClick={() => setDetailsOpen(false)}></div>
          <div className="absolute right-0 top-0 bottom-0 w-[420px] bg-gray-950 border-l border-white/10 p-6 overflow-y-auto">
            <div className="flex items-center justify-between mb-4">
              <div className="text-xs text-gold/70 uppercase tracking-widest">推演明细</div>
              <button onClick={() => setDetailsOpen(false)} className="text-[10px] text-gray-400 hover:text-white">关闭</button>
            </div>
            <div className="text-[10px] text-gray-400 mb-2">Raw Mapping Record</div>
            <pre className="text-[10px] bg-black/40 p-3 rounded border border-white/5 overflow-auto">{JSON.stringify(data.mapping_record || {}, null, 2)}</pre>
            {data.period_info && (
              <div className="mt-4 space-y-3">
                <div className="text-[10px] text-gray-400">Segments</div>
                <div className="grid grid-cols-2 gap-2 text-[12px]">
                  <div className="bg-black/30 p-2 rounded border border-white/5">
                    <div className="text-gray-500">会</div>
                    <div>{data.period_info.hui.name} · {data.period_info.hui.start_year} – {data.period_info.hui.end_year}</div>
                  </div>
                  <div className="bg-black/30 p-2 rounded border border-white/5">
                    <div className="text-gray-500">运</div>
                    <div>{data.period_info.yun.name} · {data.period_info.yun.start_year} – {data.period_info.yun.end_year}</div>
                  </div>
                  <div className="bg-black/30 p-2 rounded border border-white/5">
                    <div className="text-gray-500">世</div>
                    <div>{data.period_info.shi.name} · {data.period_info.shi.start_year} – {data.period_info.shi.end_year}</div>
                  </div>
                  <div className="bg-black/30 p-2 rounded border border-white/5">
                    <div className="text-gray-500">旬</div>
                    <div>{data.period_info.xun.name} · {data.period_info.xun.start_year} – {data.period_info.xun.end_year}</div>
                  </div>
                </div>
              </div>
            )}
          </div>
        </div>
      )}
    </div>
  );
};

const Chip: React.FC<{ label: string; value: string; color: string }> = ({ label, value, color }) => (
  <div className="flex items-center bg-white/5 rounded-full px-3 py-1 border border-white/5 hover:bg-white/10 transition-colors cursor-help group relative whitespace-nowrap">
    <span className="text-[10px] text-gray-500 mr-2 uppercase tracking-wider shrink-0">{label}</span>
    <span className={`text-sm font-serif ${color} shrink-0`}>{value}</span>
    
    {/* Simple Tooltip */}
    <div className="absolute bottom-full left-1/2 -translate-x-1/2 mb-2 hidden group-hover:block w-max p-2 bg-gray-900 text-[10px] text-gray-300 rounded border border-white/10 shadow-xl z-50 text-center pointer-events-none">
      {getTooltipText(label)}
    </div>
  </div>
);

function getTooltipText(label: string): string {
  switch (label) {
    case '元': return 'Epoch: 129600 years';
    case '会': return 'Era: 10800 years';
    case '运': return 'Cycle: 360 years';
    case '世': return 'Generation: 30 years';
    case '旬': return 'Decade: 10 years';
    default: return '';
  }
}

export default FortuneCard;
