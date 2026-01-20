/**
 * BaziChartView - 八字四柱展示组件
 * 展示年柱、月柱、日柱（日主高亮）、时柱
 */

import { useState, useEffect, useRef } from 'react';
import { createPortal } from 'react-dom';
import { Info, ChevronRight, Sparkles } from 'lucide-react';

export interface BaziResult {
  // 四柱
  year_pillar: Pillar;
  month_pillar: Pillar;
  day_pillar: Pillar;   // 日主
  hour_pillar: Pillar;
  
  // 五行分析
  wuxing_analysis: {
    day_master: string;        // 日主五行，如 "阳木"
    day_master_gan?: string;   // 日主天干，如 "甲"
    day_master_strength: 'strong' | 'weak' | 'balanced';  // 旺/弱/中和
    wuxing_counts: Record<string, number>;  // 五行统计
    missing_wuxing: string[];   // 缺失的五行
  };
  
  // 十神分析
  ten_gods_summary?: {
    year_gan: string;
    month_gan: string;
    day_gan: string;
    hour_gan: string;
  };
  
  // 大运
  dayun?: DayunCycle[];
  
  // 小运
  xiaoyun?: XiaoyunCycle;
  
  // 流年
  liunian?: LiunianYear[];
  
  // 其他信息
  gender: 'male' | 'female' | 'other';
  birth_year?: number;
  solar_term?: string;
  zodiac?: string;
}

interface Pillar {
  gan: string;       // 天干
  zhi: string;       // 地支
  gan_wuxing: string;  // 天干五行
  zhi_wuxing: string;  // 地支五行
  zhi_animal: string;  // 生肖
  nayin: string;       // 纳音
  gan_ten_god?: string;  // 天干十神
  hidden_stems?: HiddenStem[];  // 地支藏干
}

interface HiddenStem {
  gan: string;
  gan_wuxing: string;
  ten_god: string;
  type: string;  // 余气、中气、本气
  energy: number;  // 能量百分比
}

interface DayunCycle {
  cycle: number;
  gan: string;
  zhi: string;
  gan_wuxing: string;
  zhi_wuxing: string;
  start_age: number;
  end_age: number;
  year_range: string;
}

interface XiaoyunCycle {
  age: number;
  year: number;
  gan: string;
  zhi: string;
  gan_wuxing: string;
  zhi_wuxing: string;
}

interface LiunianYear {
  year: number;
  age: number;
  gan: string;
  zhi: string;
  gan_wuxing: string;
  zhi_wuxing: string;
  zodiac: string;
}

interface BaziChartViewProps {
  data: BaziResult | null;
  isLoading: boolean;
}

// 五行颜色映射
const WUXING_COLORS: Record<string, { bg: string; text: string; border: string; glow: string }> = {
  '木': { bg: 'bg-emerald-500/20', text: 'text-emerald-300', border: 'border-emerald-500/40', glow: 'shadow-emerald-500/20' },
  '火': { bg: 'bg-red-500/20', text: 'text-red-300', border: 'border-red-500/40', glow: 'shadow-red-500/20' },
  '土': { bg: 'bg-amber-500/20', text: 'text-amber-300', border: 'border-amber-500/40', glow: 'shadow-amber-500/20' },
  '金': { bg: 'bg-slate-300/20', text: 'text-slate-200', border: 'border-slate-300/40', glow: 'shadow-slate-300/20' },
  '水': { bg: 'bg-blue-500/20', text: 'text-blue-300', border: 'border-blue-500/40', glow: 'shadow-blue-500/20' },
};

// 获取五行（从"阳木"等提取"木"）
const extractWuxing = (s: string) => s.replace(/[阴阳]/g, '');

export default function BaziChartView({ data, isLoading }: BaziChartViewProps) {
  const [showDetail, setShowDetail] = useState(false);
  const mounted = typeof document !== 'undefined';
  const hideTimerRef = useRef<number | null>(null);

  // 安全的关闭函数，带防抖（需要在 effect 里引用，故提前声明）
  const handleClose = () => {
    if (hideTimerRef.current) {
      clearTimeout(hideTimerRef.current);
    }
    setShowDetail(false);
  };

  // 安全的打开函数
  const handleOpen = () => {
    if (hideTimerRef.current) {
      clearTimeout(hideTimerRef.current);
      hideTimerRef.current = null;
    }
    setShowDetail(true);
  };

  // 确保客户端渲染
  useEffect(() => {
    return () => {
      if (hideTimerRef.current) {
        clearTimeout(hideTimerRef.current);
      }
    };
  }, []);

  // ESC键关闭模态框
  useEffect(() => {
    if (!showDetail) return;
    
    const handleEscape = (e: KeyboardEvent) => {
      if (e.key === 'Escape') {
        handleClose();
      }
    };
    
    document.addEventListener('keydown', handleEscape);
    // 防止背景滚动
    document.body.style.overflow = 'hidden';
    
    return () => {
      document.removeEventListener('keydown', handleEscape);
      document.body.style.overflow = '';
    };
  }, [showDetail]);

  if (isLoading) {
    return (
      <div className="flex items-center justify-center py-16">
        <div className="flex flex-col items-center gap-4">
          <div className="relative">
            <div className="w-16 h-16 rounded-full border-2 border-gold/30 border-t-gold animate-spin" />
            <Sparkles className="absolute inset-0 m-auto w-6 h-6 text-gold/50" />
          </div>
          <span className="text-sm text-gray-400">正在推算命盘...</span>
        </div>
      </div>
    );
  }

  if (!data) {
    return (
      <div className="flex flex-col items-center justify-center py-12 text-gray-500">
        <div className="w-20 h-20 rounded-full bg-white/5 flex items-center justify-center mb-4">
          <Sparkles className="w-8 h-8 opacity-30" />
        </div>
        <p className="text-sm">输入出生信息后排盘</p>
      </div>
    );
  }

  const pillars = [
    { label: '年柱', data: data.year_pillar, isMain: false },
    { label: '月柱', data: data.month_pillar, isMain: false },
    { label: '日柱', data: data.day_pillar, isMain: true },  // 日主高亮
    { label: '时柱', data: data.hour_pillar, isMain: false },
  ];

  const strengthLabel = {
    strong: '偏旺',
    weak: '偏弱',
    balanced: '中和'
  }[data.wuxing_analysis.day_master_strength];

  const strengthColor = {
    strong: 'text-emerald-400',
    weak: 'text-blue-400',
    balanced: 'text-amber-400'
  }[data.wuxing_analysis.day_master_strength];

  return (
    <div className="space-y-6">
      {/* 四柱卡片 */}
      <div className="grid grid-cols-2 sm:grid-cols-4 gap-2 sm:gap-3">
        {pillars.map((pillar, idx) => {
          const ganWuxing = extractWuxing(pillar.data.gan_wuxing);
          const colors = WUXING_COLORS[ganWuxing] || WUXING_COLORS['土'];
          
          return (
            <div
              key={pillar.label}
              className={`relative rounded-2xl border p-4 transition-all ${
                pillar.isMain
                  ? `${colors.bg} ${colors.border} shadow-lg ${colors.glow}`
                  : 'bg-white/5 border-white/10 hover:border-white/20'
              }`}
              style={{ animationDelay: `${idx * 100}ms` }}
            >
              {/* 标签 */}
              <div className="text-[10px] text-gray-500 uppercase tracking-widest mb-3 text-center">
                {pillar.label}
                {pillar.isMain && (
                  <span className="ml-1 text-gold">★</span>
                )}
              </div>
              
              {/* 干支 */}
              <div className="text-center space-y-2">
                {/* 天干 + 五行 */}
                <div>
                  <div className={`text-3xl font-bold font-serif ${colors.text}`}>
                    {pillar.data.gan}
                  </div>
                  <div className={`text-[9px] ${colors.text} opacity-70 mt-0.5`}>
                    {pillar.data.gan_wuxing}
                  </div>
                </div>
                
                {/* 地支 + 五行·生肖 */}
                <div>
                  <div className="text-3xl font-bold font-serif text-white/90">
                    {pillar.data.zhi}
                  </div>
                  <div className="text-[9px] text-gray-400 mt-0.5">
                    {pillar.data.zhi_wuxing} · {pillar.data.zhi_animal}
                  </div>
                </div>
              </div>
              
              {/* 纳音 */}
              <div className="mt-3 text-center">
                <span className="inline-block text-[8px] px-2 py-1 rounded-md bg-black/40 text-gray-400 border border-white/5 whitespace-nowrap">
                  {pillar.data.nayin}
                </span>
              </div>
              
              {/* 日主标识 */}
              {pillar.isMain && (
                <div className="absolute -top-2 -right-2 w-6 h-6 rounded-full bg-gold flex items-center justify-center shadow-lg shadow-gold/30">
                  <span className="text-[10px] text-black font-bold">主</span>
                </div>
              )}
            </div>
          );
        })}
      </div>

      {/* 五行分析摘要 */}
      <div className="p-3 rounded-xl bg-white/5 border border-white/10 space-y-3">
        {/* 日主和格局 */}
        <div className="flex items-center justify-between">
          <div className="flex items-center gap-3">
            <div className="flex items-center gap-1.5">
              <span className="text-[10px] text-gray-500">日主</span>
              <span className={`text-sm font-bold ${WUXING_COLORS[extractWuxing(data.wuxing_analysis.day_master)]?.text || 'text-white'}`}>
                {data.wuxing_analysis.day_master}
              </span>
            </div>
            <div className="w-px h-3 bg-white/10" />
            <div className="flex items-center gap-1.5">
              <span className="text-[10px] text-gray-500">格局</span>
              <span className={`text-xs font-medium ${strengthColor}`}>
                {strengthLabel}
              </span>
            </div>
          </div>
          {data.wuxing_analysis.missing_wuxing.length > 0 && (
            <div className="flex items-center gap-1">
              <span className="text-[10px] text-gray-500">缺</span>
              <span className="text-xs text-red-400">
                {data.wuxing_analysis.missing_wuxing.join('')}
              </span>
            </div>
          )}
        </div>
        
        {/* 五行分布 */}
        <div className="flex items-center justify-center gap-1">
          {['木', '火', '土', '金', '水'].map(wx => {
            const count = data.wuxing_analysis.wuxing_counts[wx] || 0;
            const colors = WUXING_COLORS[wx];
            return (
              <div
                key={wx}
                className={`flex-1 h-7 rounded flex flex-col items-center justify-center text-[9px] ${
                  count > 0 ? `${colors.bg} ${colors.text}` : 'bg-white/5 text-gray-600'
                }`}
                title={`${wx}: ${count}`}
              >
                <span className="font-bold">{count}</span>
                <span className="opacity-60">{wx}</span>
              </div>
            );
          })}
        </div>
      </div>

      {/* 详细分析按钮 */}
      <button
        onClick={handleOpen}
        className="w-full flex items-center justify-center gap-2 py-3 rounded-xl bg-white/5 border border-white/10 text-gray-400 hover:text-white hover:border-white/20 transition-all group"
        type="button"
      >
        <Info className="w-4 h-4" />
        <span className="text-sm">详细分析</span>
        <ChevronRight className="w-4 h-4 group-hover:translate-x-1 transition-transform" />
      </button>

      {/* 详细分析 Modal - 使用 Portal 渲染到 body，避免影响触发区域 */}
      {mounted && showDetail && createPortal(
        <div 
          className="fixed inset-0 z-50 flex items-center justify-center p-4"
          style={{ 
            animation: 'fadeIn 0.2s ease-out',
            willChange: 'opacity'
          }}
          onClick={handleClose}
          onMouseDown={(e) => {
            // 防止背景点击立即关闭，给内容区域一个缓冲
            if (e.target === e.currentTarget) {
              handleClose();
            }
          }}
        >
          {/* 背景遮罩 */}
          <div 
            className="absolute inset-0 bg-black/60 backdrop-blur-sm"
            aria-hidden="true"
          />
          
          {/* 内容区域 */}
          <div 
            className="relative w-full max-w-2xl max-h-[80vh] overflow-y-auto bg-[#0a0a12] border border-white/10 rounded-3xl p-6 shadow-2xl z-10"
            onClick={(e) => e.stopPropagation()}
            onMouseDown={(e) => e.stopPropagation()}
            onMouseEnter={() => {
              // 鼠标进入内容区域时，清除任何待关闭的定时器
              if (hideTimerRef.current) {
                clearTimeout(hideTimerRef.current);
                hideTimerRef.current = null;
              }
            }}
            style={{
              animation: 'slideUp 0.2s ease-out',
              willChange: 'transform, opacity'
            }}
          >
            <div className="flex items-center justify-between mb-6">
              <h3 className="text-lg font-bold text-gold">命盘详细分析</h3>
              <button
                onClick={handleClose}
                className="w-8 h-8 rounded-full bg-white/5 flex items-center justify-center text-gray-400 hover:text-white transition-colors"
                type="button"
                aria-label="关闭"
              >
                ✕
              </button>
            </div>
            
            <div className="space-y-6 text-sm text-gray-300">
              {/* 基础信息 */}
              <section>
                <h4 className="text-xs text-gray-500 uppercase tracking-widest mb-3">基础信息</h4>
                <div className="grid grid-cols-2 gap-4">
                  <div className="p-3 rounded-xl bg-white/5">
                    <span className="text-gray-500">生肖：</span>
                    <span className="ml-2">{data.zodiac || data.year_pillar.zhi_animal}</span>
                  </div>
                  <div className="p-3 rounded-xl bg-white/5">
                    <span className="text-gray-500">日主：</span>
                    <span className={`ml-2 ${WUXING_COLORS[extractWuxing(data.wuxing_analysis.day_master)]?.text}`}>
                      {data.day_pillar.gan} ({data.wuxing_analysis.day_master})
                    </span>
                  </div>
                </div>
              </section>
              
              {/* 十神分析 */}
              {data.ten_gods_summary && (
                <section>
                  <h4 className="text-xs text-gray-500 uppercase tracking-widest mb-3">十神分析</h4>
                  <div className="grid grid-cols-4 gap-2">
                    {[
                      { label: '年柱', god: data.ten_gods_summary.year_gan, pillar: data.year_pillar },
                      { label: '月柱', god: data.ten_gods_summary.month_gan, pillar: data.month_pillar },
                      { label: '日柱', god: data.ten_gods_summary.day_gan, pillar: data.day_pillar },
                      { label: '时柱', god: data.ten_gods_summary.hour_gan, pillar: data.hour_pillar }
                    ].map(item => (
                      <div key={item.label} className="p-3 rounded-xl bg-white/5 text-center">
                        <div className="text-xs text-gray-500 mb-1">{item.label}</div>
                        <div className="text-lg font-bold mb-1">{item.pillar.gan}</div>
                        <div className="text-xs text-cyan-400">{item.god}</div>
                      </div>
                    ))}
                  </div>
                  
                  {/* 地支藏干 */}
                  <div className="mt-4 p-3 rounded-xl bg-white/5">
                    <div className="text-xs text-gray-500 mb-2">地支藏干</div>
                    <div className="grid grid-cols-4 gap-2">
                      {[data.year_pillar, data.month_pillar, data.day_pillar, data.hour_pillar].map((pillar, idx) => (
                        <div key={idx} className="text-center">
                          <div className="text-xs font-bold mb-1">{pillar.zhi}</div>
                          {pillar.hidden_stems?.map((hs, hidx) => (
                            <div key={hidx} className="text-[10px] text-gray-400">
                              {hs.gan}·{hs.ten_god}
                            </div>
                          ))}
                        </div>
                      ))}
                    </div>
                  </div>
                </section>
              )}
              
              {/* 五行详解 */}
              <section>
                <h4 className="text-xs text-gray-500 uppercase tracking-widest mb-3">五行分布</h4>
                <div className="space-y-2">
                  {['木', '火', '土', '金', '水'].map(wx => {
                    const count = data.wuxing_analysis.wuxing_counts[wx] || 0;
                    const max = Math.max(...Object.values(data.wuxing_analysis.wuxing_counts));
                    const pct = max > 0 ? (count / max) * 100 : 0;
                    const colors = WUXING_COLORS[wx];
                    return (
                      <div key={wx} className="flex items-center gap-3">
                        <span className={`w-8 text-center font-bold ${colors.text}`}>{wx}</span>
                        <div className="flex-1 h-2 rounded-full bg-white/5 overflow-hidden">
                          <div
                            className={`h-full rounded-full ${colors.bg} transition-all duration-500`}
                            style={{ width: `${pct}%` }}
                          />
                        </div>
                        <span className="w-6 text-right text-gray-500">{count}</span>
                      </div>
                    );
                  })}
                </div>
              </section>
              
              {/* 大运分析 */}
              {data.dayun && data.dayun.length > 0 && (
                <section>
                  <h4 className="text-xs text-gray-500 uppercase tracking-widest mb-3">大运分析 (十年一运)</h4>
                  <div className="space-y-2 max-h-64 overflow-y-auto">
                    {data.dayun.map((cycle) => {
                      const ganWx = extractWuxing(cycle.gan_wuxing);
                      const colors = WUXING_COLORS[ganWx] || WUXING_COLORS['土'];
                      return (
                        <div key={cycle.cycle} className={`p-3 rounded-xl ${colors.bg} border ${colors.border}`}>
                          <div className="flex items-center justify-between">
                            <div className="flex items-center gap-3">
                              <div className="text-lg font-bold font-serif">
                                {cycle.gan}{cycle.zhi}
                              </div>
                              <div className="text-xs text-gray-400">
                                第{cycle.cycle}运
                              </div>
                            </div>
                            <div className="text-right">
                              <div className="text-xs font-medium">{cycle.start_age}-{cycle.end_age}岁</div>
                              <div className="text-[10px] text-gray-500">{cycle.year_range}</div>
                            </div>
                          </div>
                        </div>
                      );
                    })}
                  </div>
                </section>
              )}
              
              {/* 流年运势 */}
              {data.liunian && data.liunian.length > 0 && (
                <section>
                  <h4 className="text-xs text-gray-500 uppercase tracking-widest mb-3">流年运势</h4>
                  <div className="grid grid-cols-2 gap-2">
                    {data.liunian.map((year) => {
                      const ganWx = extractWuxing(year.gan_wuxing);
                      const colors = WUXING_COLORS[ganWx] || WUXING_COLORS['土'];
                      const isCurrent = year.year === new Date().getFullYear();
                      return (
                        <div 
                          key={year.year} 
                          className={`p-3 rounded-xl ${colors.bg} border ${isCurrent ? 'border-gold' : colors.border}`}
                        >
                          <div className="flex items-center justify-between mb-1">
                            <div className="text-xs font-bold">{year.year}年</div>
                            {isCurrent && <span className="text-[9px] px-1.5 py-0.5 rounded bg-gold text-black">当前</span>}
                          </div>
                          <div className="text-lg font-serif mb-1">{year.gan}{year.zhi}</div>
                          <div className="text-[10px] text-gray-400">
                            {year.zodiac}年 · {year.age}岁
                          </div>
                        </div>
                      );
                    })}
                  </div>
                </section>
              )}
              
              {/* 小运 */}
              {data.xiaoyun && (
                <section>
                  <h4 className="text-xs text-gray-500 uppercase tracking-widest mb-3">小运 (当前年运)</h4>
                  <div className="p-3 rounded-xl bg-white/5">
                    <div className="flex items-center justify-between">
                      <div className="flex items-center gap-3">
                        <div className="text-lg font-bold font-serif">
                          {data.xiaoyun.gan}{data.xiaoyun.zhi}
                        </div>
                        <div className="text-xs text-gray-400">
                          {data.xiaoyun.year}年 · {data.xiaoyun.age}岁
                        </div>
                      </div>
                      <div className="text-xs text-gray-500">
                        {data.xiaoyun.gan_wuxing}
                      </div>
                    </div>
                  </div>
                </section>
              )}
              
              {/* 纳音 */}
              <section>
                <h4 className="text-xs text-gray-500 uppercase tracking-widest mb-3">纳音五行</h4>
                <div className="grid grid-cols-4 gap-2">
                  {pillars.map(p => (
                    <div key={p.label} className="text-center p-2 rounded-lg bg-white/5">
                      <div className="text-xs text-gray-500 mb-1">{p.label}</div>
                      <div className="text-sm">{p.data.nayin}</div>
                    </div>
                  ))}
                </div>
              </section>
              
              {/* 提示 */}
              <div className="p-4 rounded-xl bg-emerald-500/10 border border-emerald-500/20 text-emerald-200/80 text-xs">
                <strong className="text-emerald-300">✓ 已完善：</strong> 本系统已集成十神、大运、流年、小运等传统八字分析要素，提供全面的命理解读。
              </div>
            </div>
          </div>
        </div>,
        document.body
      )}
    </div>
  );
}

