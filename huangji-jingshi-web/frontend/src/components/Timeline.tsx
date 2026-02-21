import React, { useEffect, useState } from 'react';
import type { FortuneResponse, PeriodInfo, TimelineLevel, TimelineResponse } from '../types';

interface TimelineProps {
  currentYear: number;
  currentDatetime: string;
  onYearChange: (year: number) => void;
  mapping?: FortuneResponse['mapping_record'];
  mode: 'algorithm' | 'table' | 'compare';
  yearStart: 'lichun' | 'gregorian';
  primary: 'algorithm' | 'table';
  tzOffsetMinutes?: number;
  lon?: number;
}

interface TimelineEvent {
  year: number;
  title: string;
  description?: string;
}

interface SelectedNode {
  level: TimelineLevel;
  name: string;
  index: number;
  start_year: number;
  end_year: number;
  max_index: number;
}

type SelectedPeriod = PeriodInfo & { level: TimelineLevel };

type MobilePanel = 'detail' | 'critical' | 'events';

interface TransitionItem {
  level: TimelineLevel;
  startYear: number;
  targetName: string;
  index: number;
  maxIndex: number;
}

const LEVEL_META: Record<
  TimelineLevel,
  {
    label: string;
    enLabel: string;
    colorText: string;
    colorSoftBg: string;
    colorBorder: string;
    colorBar: string;
  }
> = {
  hui: {
    label: '会',
    enLabel: 'ERA',
    colorText: 'text-cyan-300',
    colorSoftBg: 'bg-cyan-500/10',
    colorBorder: 'border-cyan-500/30',
    colorBar: 'bg-cyan-400',
  },
  yun: {
    label: '运',
    enLabel: 'CYCLE',
    colorText: 'text-emerald-300',
    colorSoftBg: 'bg-emerald-500/10',
    colorBorder: 'border-emerald-500/30',
    colorBar: 'bg-emerald-400',
  },
  shi: {
    label: '世',
    enLabel: 'GENERATION',
    colorText: 'text-amber-300',
    colorSoftBg: 'bg-amber-500/10',
    colorBorder: 'border-amber-500/30',
    colorBar: 'bg-amber-400',
  },
  xun: {
    label: '旬',
    enLabel: 'DECADE',
    colorText: 'text-purple-300',
    colorSoftBg: 'bg-purple-500/10',
    colorBorder: 'border-purple-500/30',
    colorBar: 'bg-purple-400',
  },
};

const XUN_NAMES = ['甲子', '甲戌', '甲申'];

function getListByLevel(data: TimelineResponse, level: TimelineLevel): PeriodInfo[] {
  switch (level) {
    case 'hui':
      return data.hui_list;
    case 'yun':
      return data.yun_list;
    case 'shi':
      return data.shi_list;
    case 'xun':
      return data.xun_list;
    default:
      return data.xun_list;
  }
}

function getCurrentByLevel(data: TimelineResponse, level: TimelineLevel): PeriodInfo {
  switch (level) {
    case 'hui':
      return data.current.hui;
    case 'yun':
      return data.current.yun;
    case 'shi':
      return data.current.shi;
    case 'xun':
      return data.current.xun;
    default:
      return data.current.xun;
  }
}

function periodToSelected(level: TimelineLevel, period: PeriodInfo): SelectedNode {
  return {
    level,
    name: period.name,
    index: period.index,
    start_year: period.start_year,
    end_year: period.end_year,
    max_index: period.max_index,
  };
}

function formatNodeName(level: TimelineLevel, name: string): string {
  if (level === 'hui') return `${name}会`;
  if (level === 'yun') return `${name}运`;
  if (level === 'shi') return `${name}世`;
  return `${name}旬`;
}

function toTzLabel(offsetMinutes: number): string {
  const sign = offsetMinutes >= 0 ? '+' : '-';
  const absMinutes = Math.abs(offsetMinutes);
  const hours = String(Math.floor(absMinutes / 60)).padStart(2, '0');
  const minutes = String(absMinutes % 60).padStart(2, '0');
  return `UTC${sign}${hours}:${minutes}`;
}

function formatWithOffset(datetimeIso: string, offsetMinutes: number): string {
  const dt = new Date(datetimeIso);
  if (Number.isNaN(dt.getTime())) return '--';
  const shifted = new Date(dt.getTime() + offsetMinutes * 60_000);
  const y = shifted.getUTCFullYear();
  const m = String(shifted.getUTCMonth() + 1).padStart(2, '0');
  const d = String(shifted.getUTCDate()).padStart(2, '0');
  const hh = String(shifted.getUTCHours()).padStart(2, '0');
  const mm = String(shifted.getUTCMinutes()).padStart(2, '0');
  return `${y}-${m}-${d} ${hh}:${mm}`;
}

function resolveSelectedPeriod(
  data: TimelineResponse,
  selectedNode: SelectedNode | null,
): SelectedPeriod {
  if (selectedNode) {
    const list = getListByLevel(data, selectedNode.level);
    const matched =
      list.find(
        (item) => item.index === selectedNode.index && item.start_year === selectedNode.start_year,
      ) || list.find((item) => item.start_year === selectedNode.start_year);

    if (matched) {
      return {
        ...matched,
        level: selectedNode.level,
      };
    }
  }

  return {
    ...data.current.xun,
    level: 'xun',
  };
}

function buildNextTransitions(data: TimelineResponse): TransitionItem[] {
  const levels: TimelineLevel[] = ['xun', 'shi', 'yun', 'hui'];

  return levels.map((level) => {
    const current = getCurrentByLevel(data, level);
    const list = getListByLevel(data, level);

    const currentPos = list.findIndex(
      (item) => item.index === current.index && item.start_year === current.start_year,
    );
    const nextInList =
      currentPos >= 0 && currentPos < list.length - 1 ? list[currentPos + 1] : undefined;

    if (nextInList) {
      return {
        level,
        startYear: nextInList.start_year,
        targetName: nextInList.name,
        index: nextInList.index,
        maxIndex: nextInList.max_index,
      };
    }

    if (level === 'xun') {
      const xunIdx = XUN_NAMES.indexOf(current.name);
      const nextName =
        xunIdx >= 0 ? XUN_NAMES[(xunIdx + 1) % XUN_NAMES.length] : list[0]?.name || XUN_NAMES[0];
      return {
        level,
        startYear: current.end_year + 1,
        targetName: nextName,
        index: 1,
        maxIndex: current.max_index,
      };
    }

    return {
      level,
      startYear: current.end_year + 1,
      targetName: `下一${LEVEL_META[level].label}`,
      index: 1,
      maxIndex: current.max_index,
    };
  });
}

const Timeline: React.FC<TimelineProps> = ({
  currentYear,
  currentDatetime,
  onYearChange,
  mapping,
  mode,
  yearStart,
  primary,
  tzOffsetMinutes,
  lon,
}) => {
  const API_BASE = import.meta.env.VITE_BACKEND_URL || '';
  const [data, setData] = useState<TimelineResponse | null>(null);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [showHelp, setShowHelp] = useState(false);
  const [events, setEvents] = useState<TimelineEvent[]>([]);
  const [huiDoc, setHuiDoc] = useState<string | null>(null);
  const [selectedNode, setSelectedNode] = useState<SelectedNode | null>(null);
  const [mobilePanel, setMobilePanel] = useState<MobilePanel>('detail');

  useEffect(() => {
    const fetchData = async () => {
      setLoading(true);
      try {
        const query = new URLSearchParams({
          datetime: currentDatetime,
          mode,
          yearStart,
          primary,
          tzOffsetMinutes: String(tzOffsetMinutes ?? 480),
          lon: String(lon ?? 116.4),
        });
        const response = await fetch(`${API_BASE}/api/timeline?${query}`);
        if (!response.ok) throw new Error('Failed to fetch timeline');
        const json = (await response.json()) as TimelineResponse;
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
  }, [currentDatetime, API_BASE, mode, yearStart, primary, tzOffsetMinutes, lon]);

  useEffect(() => {
    if (!data) return;

    setSelectedNode((prev) => {
      if (prev) {
        const list = getListByLevel(data, prev.level);
        const found = list.find(
          (item) => item.start_year === prev.start_year && item.index === prev.index,
        );
        if (found) {
          return periodToSelected(prev.level, found);
        }
      }
      return periodToSelected('xun', data.current.xun);
    });
  }, [data]);

  useEffect(() => {
    if (!data) return;
    const start = data.current.hui.start_year;
    const end = data.current.hui.end_year;
    const run = async () => {
      try {
        const query = new URLSearchParams({
          year: String(start),
          mode,
          yearStart,
          primary,
        });
        const r = await fetch(`${API_BASE}/api/mapping/get?${query}`);
        const j = await r.json();
        const hex =
          (j.record_normalized?.nian_hexagram as string | undefined) ||
          (j.record_raw?.nian_hexagram as string | undefined);
        if (hex) setHuiDoc(`${hex}（${start}–${end}）`);
        else setHuiDoc(null);
      } catch {
        setHuiDoc(null);
      }
    };
    run();
  }, [data, API_BASE, mode, yearStart, primary]);

  useEffect(() => {
    if (!data) return;

    const rangeFor = (list: PeriodInfo[], activeIndex: number) => {
      const pos = list.findIndex((item) => item.index === activeIndex);
      const segs = [
        ...(pos > 0 ? [list[pos - 1]] : []),
        ...(pos >= 0 ? [list[pos]] : []),
        ...(pos >= 0 && pos < list.length - 1 ? [list[pos + 1]] : []),
      ];
      return {
        start: Math.min(...segs.map((item) => item.start_year)),
        end: Math.max(...segs.map((item) => item.end_year)),
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
        if (!resp.ok) throw new Error('Failed history fetch');
        const json = await resp.json();
        if (!Array.isArray(json)) {
          setEvents([]);
          return;
        }
        const normalized: TimelineEvent[] = json
          .filter((item): item is Record<string, unknown> => !!item && typeof item === 'object')
          .map((item) => ({
            year: Number(item.year),
            title: typeof item.title === 'string' ? item.title : '未命名事件',
            description: typeof item.description === 'string' ? item.description : undefined,
          }))
          .filter((item) => Number.isFinite(item.year));
        setEvents(normalized);
      } catch {
        setEvents([]);
      }
    };

    fetchEvents();
  }, [data, API_BASE]);

  if (loading && !data) return <div className="h-32 bg-white/5 animate-pulse rounded-lg" />;
  if (error)
    return <div className="text-red-400 text-xs p-4 border border-red-900 rounded-lg">{error}</div>;
  if (!data) return null;

  const offsetMinutes =
    typeof tzOffsetMinutes === 'number' && Number.isFinite(tzOffsetMinutes)
      ? tzOffsetMinutes
      : -new Date(currentDatetime).getTimezoneOffset();
  const tzLabel = toTzLabel(offsetMinutes);
  const localStr = formatWithOffset(currentDatetime, offsetMinutes);

  const selectedPeriod = resolveSelectedPeriod(data, selectedNode);
  const selectedMeta = LEVEL_META[selectedPeriod.level];

  const spanYears = Math.max(1, selectedPeriod.end_year - selectedPeriod.start_year + 1);
  const elapsedYears = Math.max(0, Math.min(data.year - selectedPeriod.start_year, spanYears));
  const remainingYears = Math.max(0, selectedPeriod.end_year - data.year);
  const progress = Math.max(0, Math.min(100, (elapsedYears / spanYears) * 100));

  const selectedEvents = events
    .filter(
      (event) => event.year >= selectedPeriod.start_year && event.year <= selectedPeriod.end_year,
    )
    .sort((a, b) => a.year - b.year)
    .slice(0, 6);

  const transitions = buildNextTransitions(data);

  const renderEvents = (period: PeriodInfo) => {
    const eventsInPeriod = events.filter(
      (event) => event.year >= period.start_year && event.year <= period.end_year,
    );
    if (eventsInPeriod.length === 0) return null;

    return (
      <div className="absolute bottom-0 left-0 right-0 flex justify-center space-x-1 pb-1">
        {eventsInPeriod.map((event) => (
          <div
            key={`${event.year}-${event.title}`}
            className="w-1 h-1 bg-red-500 rounded-full"
            title={`${event.year}: ${event.title}`}
          />
        ))}
      </div>
    );
  };

  const handleSelectPeriod = (level: TimelineLevel, item: PeriodInfo) => {
    setSelectedNode(periodToSelected(level, item));
    onYearChange(item.start_year);
  };

  const handleTransitionJump = (item: TransitionItem) => {
    setSelectedNode({
      level: item.level,
      name: item.targetName,
      index: item.index,
      start_year: item.startYear,
      end_year: item.startYear,
      max_index: item.maxIndex,
    });
    onYearChange(item.startYear);
  };

  const renderRow = (
    title: string,
    level: TimelineLevel,
    items: PeriodInfo[],
    activeIndex: number,
    colorClass: string,
  ) => {
    let centerPos = items.findIndex((item) => item.index === activeIndex);
    if (centerPos < 0 && items.length > 0)
      centerPos = Math.min(items.length - 1, Math.floor(items.length / 2));

    const displayItems: PeriodInfo[] = [];
    if (centerPos > 0) displayItems.push(items[centerPos - 1]);
    if (centerPos >= 0 && centerPos < items.length) displayItems.push(items[centerPos]);
    if (centerPos >= 0 && centerPos < items.length - 1) displayItems.push(items[centerPos + 1]);

    let activeItem = items.find((item) => item.index === activeIndex);
    if (!activeItem && centerPos >= 0 && items.length > 0) activeItem = items[centerPos];

    const typeLabel = LEVEL_META[level].label;
    const spanLabel =
      level === 'hui' ? '10800年' : level === 'yun' ? '360年' : level === 'shi' ? '30年' : '10年';

    return (
      <div className="mb-6" key={level}>
        <div className="flex items-center justify-between mb-2">
          <div className="relative group">
            <h4 className="text-[10px] text-gray-400 uppercase tracking-widest w-24 shrink-0">
              {title}
            </h4>
            <div className="absolute right-full top-1/2 -translate-y-1/2 mr-2 w-64 bg-black/90 border border-white/20 rounded p-2 hidden group-hover:block z-20 pointer-events-none">
              <div className="text-[10px] text-gray-400 mb-1 text-center">
                {typeLabel} · {spanLabel}
              </div>
              {activeItem && (
                <div className="text-[10px] text-gray-200 text-center">当前：{activeItem.name}</div>
              )}
              {activeItem && (
                <div className="text-[10px] text-gray-500 text-center">
                  公元 {activeItem.start_year} – {activeItem.end_year}
                </div>
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
            const isSelected =
              selectedPeriod.level === level &&
              selectedPeriod.start_year === item.start_year &&
              selectedPeriod.index === item.index;

            return (
              <button
                key={`${title}-${item.index}`}
                onClick={() => handleSelectPeriod(level, item)}
                className={`
                  flex-shrink-0 relative group transition-all duration-300
                  ${isActive ? 'w-32' : 'w-20 hover:w-24'}
                  h-16 rounded-sm border overflow-hidden
                  ${isActive ? 'bg-white/10 border-white/20' : 'bg-white/5 hover:bg-white/10 border-white/5'}
                  ${isSelected ? 'ring-1 ring-gold/70' : ''}
                `}
              >
                <div
                  className={`absolute inset-0 opacity-20 ${isActive ? colorClass : isPast ? 'bg-gray-800' : ''}`}
                />

                {isActive && (
                  <div
                    className={`absolute top-0 left-0 right-0 h-0.5 ${colorClass} shadow-[0_0_8px_currentColor]`}
                  />
                )}

                <div className="absolute inset-0 p-2 flex flex-col justify-between">
                  <span
                    className={`text-[10px] font-mono truncate ${isActive ? 'text-white' : 'text-gray-500 group-hover:text-gray-300'}`}
                  >
                    {formatNodeName(level, item.name)}
                  </span>

                  <span className="text-[9px] text-gray-600 font-mono absolute bottom-1 right-2 opacity-50 group-hover:opacity-100">
                    {item.index}
                  </span>

                  {isActive && (
                    <div className="text-[9px] text-gray-400 font-mono mt-1">{item.start_year}</div>
                  )}

                  {renderEvents(item)}
                </div>

                <div className="absolute bottom-full left-1/2 -translate-x-1/2 mb-2 w-56 bg-black/90 border border-white/20 rounded p-2 hidden group-hover:block z-20 pointer-events-none">
                  <div className="text-[10px] text-gray-400 mb-1 text-center">
                    公元 {item.start_year} – {item.end_year} · {formatNodeName(level, item.name)}
                  </div>
                  {level === 'yun' && (
                    <div className="text-[10px] text-gray-200 text-center">
                      {mapping?.yun_raw
                        ? mapping.yun_raw
                        : `${item.name}（${item.start_year}-${item.end_year}）`}
                    </div>
                  )}
                  {level === 'hui' && (
                    <div className="text-[10px] text-gray-200 text-center">
                      {(mapping?.hui_raw || `${item.name}会`) +
                        (item.index === activeIndex && huiDoc ? ` · ${huiDoc}` : '')}
                    </div>
                  )}
                  {level === 'xun' && (
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

  const detailCard = (
    <div className={`bg-black/30 border rounded-2xl p-4 space-y-3 ${selectedMeta.colorBorder}`}>
      <div className="flex items-center justify-between">
        <div>
          <div className="text-[10px] text-gray-500 uppercase tracking-widest">
            当前选中节点详情
          </div>
          <div className={`text-sm font-serif mt-1 ${selectedMeta.colorText}`}>
            {formatNodeName(selectedPeriod.level, selectedPeriod.name)}
          </div>
        </div>
        <div className="text-right">
          <div className="text-[10px] text-gray-500">层级</div>
          <div className="text-[11px] text-gray-300">
            {selectedMeta.label} · {selectedMeta.enLabel}
          </div>
        </div>
      </div>

      <div className="text-[11px] text-gray-400 leading-relaxed">
        公历范围：{selectedPeriod.start_year}-01-01 00:00 ~ {selectedPeriod.end_year}-12-31 23:59{' '}
        {tzLabel}
      </div>

      <div className="space-y-2">
        <div className="flex items-center justify-between text-[11px]">
          <span className="text-gray-500">层级进度</span>
          <span className="text-gray-300">
            已过 {elapsedYears} 年 · 剩余 {remainingYears} 年
          </span>
        </div>
        <div className="h-1.5 rounded-full bg-white/10 overflow-hidden">
          <div
            className={`h-full transition-all duration-300 ${selectedMeta.colorBar}`}
            style={{ width: `${progress}%` }}
          ></div>
        </div>
      </div>

      <div className="grid grid-cols-2 gap-2 text-[11px]">
        <div className="bg-black/40 border border-white/5 rounded-lg p-2">
          <div className="text-gray-500">序号</div>
          <div className="text-gray-200">
            {selectedPeriod.index}/{selectedPeriod.max_index}
          </div>
        </div>
        <div className="bg-black/40 border border-white/5 rounded-lg p-2">
          <div className="text-gray-500">跨度</div>
          <div className="text-gray-200">{spanYears} 年</div>
        </div>
        <div className="bg-black/40 border border-white/5 rounded-lg p-2">
          <div className="text-gray-500">中心经世年</div>
          <div className="text-gray-200">{data.year}</div>
        </div>
        <div className="bg-black/40 border border-white/5 rounded-lg p-2">
          <div className="text-gray-500">当前值年卦</div>
          <div className="text-gray-200">{data.current.year_gua}</div>
        </div>
      </div>

      {data.calc_meta && (
        <div
          className={`text-[10px] rounded-lg border p-2 ${selectedMeta.colorSoftBg} ${selectedMeta.colorBorder}`}
        >
          口径：{data.calc_meta.mode} / {data.calc_meta.primary} / {data.calc_meta.year_start}
        </div>
      )}
    </div>
  );

  const criticalCard = (
    <div className="bg-black/30 border border-white/10 rounded-2xl p-4 space-y-3">
      <div className="text-[10px] text-gray-500 uppercase tracking-widest">临界点 / 倒计时</div>
      <div className="space-y-2">
        {transitions.map((item) => {
          const yearsLeft = Math.max(0, item.startYear - data.year);
          const meta = LEVEL_META[item.level];
          return (
            <button
              key={`${item.level}-${item.startYear}`}
              onClick={() => handleTransitionJump(item)}
              className="w-full bg-black/40 hover:bg-black/50 border border-white/10 hover:border-white/20 rounded-lg p-2.5 transition-colors text-left"
            >
              <div className="flex items-center justify-between">
                <div className="text-[11px] text-gray-200">
                  下一{meta.label}：{formatNodeName(item.level, item.targetName)}
                </div>
                <div className={`text-[11px] font-mono ${meta.colorText}`}>{yearsLeft}y</div>
              </div>
              <div className="text-[10px] text-gray-500 mt-1">
                切换年份：{item.startYear} · 点击跳转
              </div>
            </button>
          );
        })}
      </div>
    </div>
  );

  const eventsCard = (
    <div className="bg-black/30 border border-white/10 rounded-2xl p-4 space-y-3">
      <div className="flex items-center justify-between">
        <div className="text-[10px] text-gray-500 uppercase tracking-widest">关联事件</div>
        <div className="text-[10px] text-gray-500">
          {selectedPeriod.start_year} – {selectedPeriod.end_year}
        </div>
      </div>
      {selectedEvents.length === 0 ? (
        <div className="text-[11px] text-gray-500">当前节点范围暂无事件记录</div>
      ) : (
        <div className="space-y-2">
          {selectedEvents.map((event) => (
            <button
              key={`${event.year}-${event.title}`}
              onClick={() => onYearChange(event.year)}
              className="w-full text-left bg-black/40 hover:bg-black/50 border border-white/10 rounded-lg p-2.5 transition-colors"
              title={event.description || event.title}
            >
              <div className="text-[11px] text-gray-200 leading-snug">
                <span className="text-gold/90 mr-1">{event.year}</span>
                {event.title}
              </div>
              {event.description && (
                <div className="text-[10px] text-gray-500 mt-1 line-clamp-2">
                  {event.description}
                </div>
              )}
            </button>
          ))}
        </div>
      )}
    </div>
  );

  const renderMobilePanel = (panel: MobilePanel, title: string, content: React.ReactNode) => {
    const isOpen = mobilePanel === panel;
    return (
      <div className="border border-white/10 rounded-xl overflow-hidden bg-black/20" key={panel}>
        <button
          className="w-full px-3 py-2.5 flex items-center justify-between text-left"
          onClick={() => setMobilePanel(isOpen ? 'detail' : panel)}
        >
          <span className="text-xs text-gray-300">{title}</span>
          <span className="text-xs text-gray-500">{isOpen ? '−' : '+'}</span>
        </button>
        {isOpen && <div className="p-3 pt-0">{content}</div>}
      </div>
    );
  };

  return (
    <div
      id="timeline-section"
      className="bg-black/40 backdrop-blur-sm border border-white/10 rounded-xl p-6 w-full relative"
    >
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
            CENTER: {data.current.year_gua} ({data.year}) · {localStr} {tzLabel}
          </div>
        </div>
      </div>

      {showHelp && (
        <div
          className="fixed inset-0 z-[9999] flex items-center justify-center p-4 bg-black/60 backdrop-blur-sm animate-in fade-in duration-200"
          onClick={() => setShowHelp(false)}
        >
          <div
            className="bg-white w-full max-w-md rounded-xl shadow-2xl p-6 relative animate-in zoom-in-95 duration-200"
            onClick={(e) => e.stopPropagation()}
            style={{ backgroundColor: '#ffffff', color: '#1f2937' }}
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
                  <strong className="text-gray-900 block text-sm font-bold">
                    会 (Era) - 10,800年
                  </strong>
                  <p className="text-xs text-gray-600 mt-1 leading-relaxed">
                    宇宙的大月。共12会（子至亥）。
                    <br />
                    当前处于
                    <span className="text-cyan-600 font-medium">“{data.current.hui.name}会”</span>（
                    {data.current.hui.start_year}–{data.current.hui.end_year}）。
                  </p>
                </div>
              </li>
              <li className="flex items-start p-3 rounded-lg hover:bg-gray-50 transition-colors">
                <div className="w-3 h-3 mt-1.5 rounded-full bg-emerald-500 mr-4 shrink-0 shadow-sm"></div>
                <div>
                  <strong className="text-gray-900 block text-sm font-bold">
                    运 (Cycle) - 360年
                  </strong>
                  <p className="text-xs text-gray-600 mt-1 leading-relaxed">
                    宇宙的大日。一运360年，对应历史上大的朝代兴衰周期。
                    <br />
                    当前处于
                    <span className="text-emerald-600 font-medium">
                      “{data.current.yun.name}运”
                    </span>
                    （{data.current.yun.start_year}–{data.current.yun.end_year}）。
                  </p>
                </div>
              </li>
              <li className="flex items-start p-3 rounded-lg hover:bg-gray-50 transition-colors">
                <div className="w-3 h-3 mt-1.5 rounded-full bg-amber-500 mr-4 shrink-0 shadow-sm"></div>
                <div>
                  <strong className="text-gray-900 block text-sm font-bold">
                    世 (Generation) - 30年
                  </strong>
                  <p className="text-xs text-gray-600 mt-1 leading-relaxed">
                    古语“三十年为一世”，代表一代人的时间跨度。
                    <br />
                    当前处于
                    <span className="text-amber-600 font-medium">“{data.current.shi.name}世”</span>
                    （{data.current.shi.start_year}–{data.current.shi.end_year}）。
                  </p>
                </div>
              </li>
              <li className="flex items-start p-3 rounded-lg hover:bg-gray-50 transition-colors">
                <div className="w-3 h-3 mt-1.5 rounded-full bg-purple-500 mr-4 shrink-0 shadow-sm"></div>
                <div>
                  <strong className="text-gray-900 block text-sm font-bold">
                    旬 (Decade) - 10年
                  </strong>
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

      <div className="xl:grid xl:grid-cols-10 xl:gap-5">
        <div className="space-y-2 xl:col-span-7">
          {renderRow('会 (Era)', 'hui', data.hui_list, data.current.hui.index, 'bg-cyan-500')}
          {renderRow('运 (Cycle)', 'yun', data.yun_list, data.current.yun.index, 'bg-emerald-500')}
          {renderRow(
            '世 (Generation)',
            'shi',
            data.shi_list,
            data.current.shi.index,
            'bg-amber-500',
          )}
          {renderRow('旬 (Decade)', 'xun', data.xun_list, data.current.xun.index, 'bg-purple-500')}
        </div>

        <aside className="xl:col-span-3 mt-4 xl:mt-0">
          <div className="hidden xl:flex xl:flex-col xl:gap-3">
            {detailCard}
            {criticalCard}
            {eventsCard}
          </div>

          <div className="xl:hidden space-y-2">
            {renderMobilePanel('detail', '当前选中节点详情', detailCard)}
            {renderMobilePanel('critical', '临界点 / 倒计时', criticalCard)}
            {renderMobilePanel('events', '关联事件', eventsCard)}
          </div>
        </aside>
      </div>
    </div>
  );
};

export default Timeline;
