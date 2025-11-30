
import { useState, useEffect } from 'react'
import StarField from '../components/StarField';
import ControlPanel from '../components/ControlPanel';
import FortuneCard from '../components/FortuneCard';
import SkyCard from '../components/SkyCard';
import Timeline from '../components/Timeline';
import type { CombinedResponse } from '../types';
import { Link } from 'react-router-dom';
import { ChevronLeft } from 'lucide-react';
import { resolveTimezoneOffset, resolveTimezoneOffsetSync } from '../utils/timezone';

export default function Tools() {
  const API_BASE = import.meta.env.VITE_BACKEND_URL || '';
  const [data, setData] = useState<CombinedResponse | null>(null);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  
  // Initialize state from URL or defaults
  const [params, setParams] = useState(() => {
    const search = new URLSearchParams(window.location.search);
    // Default to current time if not provided
    const dt = search.get('datetime') || new Date().toISOString();
    // Default to Beijing if not provided
    const lat = parseFloat(search.get('lat') || '39.9042');
    const lon = parseFloat(search.get('lon') || '116.4074');
    return { datetime: dt, lat, lon };
  });
  const [compareMode, setCompareMode] = useState(false);
  const [compareDatetime, setCompareDatetime] = useState<string | null>(null);
  const [compareData, setCompareData] = useState<CombinedResponse | null>(null);

  // Initialize compare mode from URL once
  useEffect(() => {
    const sp = new URLSearchParams(window.location.search);
    if (sp.get('cmp') === '1') setCompareMode(true);
    const cd = sp.get('cmp_dt');
    if (cd) setCompareDatetime(cd);
  }, []);

  // Fetch data when params or compare mode change
  useEffect(() => {
    // Update shareable URL
    const sp = new URLSearchParams(window.location.search);
    sp.set('datetime', params.datetime);
    sp.set('lat', params.lat.toString());
    sp.set('lon', params.lon.toString());
    if (compareMode) {
      sp.set('cmp', '1');
      if (compareDatetime) sp.set('cmp_dt', compareDatetime);
    } else {
      sp.delete('cmp');
      sp.delete('cmp_dt');
    }
    const url = `${window.location.pathname}?${sp.toString()}`;
    window.history.replaceState({}, '', url);

    const fetchData = async () => {
      setLoading(true);
      setError(null);
      try {
        const q = new URLSearchParams({
          datetime: params.datetime,
          lat: params.lat.toString(),
          lon: params.lon.toString()
        });
        
        const res = await fetch(`${API_BASE}/api/sky-and-fortune?${q}`);
        if (!res.ok) {
          const errText = await res.text();
          throw new Error(errText || "Server Error");
        }
        const jsonData = await res.json();
        setData(jsonData);
        if (compareMode && compareDatetime) {
          const q2 = new URLSearchParams({
            datetime: compareDatetime,
            lat: params.lat.toString(),
            lon: params.lon.toString()
          });
          const res2 = await fetch(`${API_BASE}/api/sky-and-fortune?${q2}`);
          if (res2.ok) {
            const j2 = await res2.json();
            setCompareData(j2);
          } else {
            setCompareData(null);
          }
        } else {
          setCompareData(null);
        }
      } catch (err: unknown) {
        console.error(err);
        const msg = err instanceof Error ? err.message : '请求失败，请检查网络或参数';
        setError(msg);
      } finally {
        setLoading(false);
      }
    };

    fetchData();
  }, [params, compareMode, compareDatetime, API_BASE]);

  const handleCalculate = (newParams: { datetime: string; lat: number; lon: number }) => {
    setParams(newParams);
  };

  const handleTimelineYearChange = (year: number) => {
    const current = new Date(params.datetime);
    current.setFullYear(year);
    setParams(prev => ({ ...prev, datetime: current.toISOString() }));
  };

  const handleJumpToYear = (year: number) => {
    const target = new Date(params.datetime);
    target.setFullYear(year);
    target.setMonth(0);
    target.setDate(1);
    target.setHours(12, 0, 0, 0);
    setParams(prev => ({ ...prev, datetime: target.toISOString() }));
    const el = document.getElementById('timeline-section');
    if (el) {
      el.scrollIntoView({ behavior: 'smooth', block: 'start' });
    }
  };

  // Calculate timezone offset for target location and datetime
  const initialTz = resolveTimezoneOffsetSync(new Date(params.datetime), params.lat, params.lon).offsetHours;
  const [timezoneOffset, setTimezoneOffset] = useState<number>(initialTz);
  const [compareTimezoneOffset, setCompareTimezoneOffset] = useState<number | null>(null);

  useEffect(() => {
    const sp = new URLSearchParams(window.location.search);
    const preferRemote = sp.get('tz') === 'remote';
    const mainDate = new Date(params.datetime);
    resolveTimezoneOffset(mainDate, params.lat, params.lon, preferRemote)
      .then((r) => setTimezoneOffset(r.offsetHours))
      .catch(() => setTimezoneOffset(0));
    if (compareMode && compareDatetime) {
      const cmpDate = new Date(compareDatetime);
      resolveTimezoneOffset(cmpDate, params.lat, params.lon, preferRemote)
        .then((r) => setCompareTimezoneOffset(r.offsetHours))
        .catch(() => setCompareTimezoneOffset(null));
    } else {
      setCompareTimezoneOffset(null);
    }
  }, [params, compareMode, compareDatetime]);

  const [debugOpen, setDebugOpen] = useState(false);

  useEffect(() => {
    const search = new URLSearchParams(window.location.search);
    if (search.get('debug') === '1') setDebugOpen(true);
    const onKey = (e: KeyboardEvent) => {
      if ((e.ctrlKey || e.metaKey) && e.key.toLowerCase() === 'd') {
        setDebugOpen(v => !v);
      }
    };
    window.addEventListener('keydown', onKey);
    return () => window.removeEventListener('keydown', onKey);
  }, []);

  return (
    <div className="relative min-h-screen bg-[#050508] text-white font-sans overflow-x-hidden">
      
      {/* Nebula Background */}
      <div className="nebula-container">
        <div className="nebula-layer nebula-1"></div>
        <div className="nebula-layer nebula-2"></div>
        <div className="nebula-layer nebula-3"></div>
      </div>

      {/* StarField Overlay */}
      <div className="fixed inset-0 z-0 opacity-60 mix-blend-screen pointer-events-none">
        <StarField />
      </div>

      {/* Main Layout Container */}
      <div className="relative z-10 max-w-[1600px] mx-auto p-4 md:p-6">
        
        {/* Header / Branding */}
        <header className="mb-8 flex items-center justify-between glass-panel rounded-2xl px-6 py-4">
           <div className="flex items-center gap-4">
              <Link to="/" className="flex items-center gap-2 text-gray-400 hover:text-white transition-colors">
                <ChevronLeft className="w-5 h-5" />
              </Link>
              <div className="w-px h-6 bg-white/10"></div>
              <div className="flex items-center gap-3">
                 <h1 className="text-xl md:text-2xl font-serif font-bold text-transparent bg-clip-text bg-gradient-to-r from-gold via-yellow-200 to-gold tracking-[0.2em]">
                   推演计算
                 </h1>
                 <span className="hidden sm:inline-block text-[10px] text-gray-500 px-2 border-l border-white/10 uppercase tracking-widest">
                   Cosmic Calculator
                 </span>
              </div>
           </div>
           <div className="flex items-center gap-3">
              <button 
                onClick={() => setCompareMode(m => !m)}
                className={`text-xs px-3 py-1.5 rounded-full border transition-all ${
                  compareMode 
                    ? 'bg-gold/20 border-gold text-gold' 
                    : 'bg-white/5 border-white/10 text-gray-400 hover:text-white hover:border-white/30'
                }`}
              >
                {compareMode ? '关闭对比' : '对比模式'}
              </button>
              {compareMode && (
                <input 
                  type="date"
                  value={compareDatetime ? compareDatetime.slice(0,10) : ''}
                  onChange={(e) => {
                    const d = new Date(e.target.value + 'T12:00:00Z').toISOString();
                    setCompareDatetime(d);
                  }}
                  className="bg-black/30 border border-white/10 text-xs text-white px-3 py-1.5 rounded-full focus:outline-none focus:border-gold/50 transition-colors"
                />
              )}
           </div>
        </header>

        {/* Content Grid */}
        <div className="grid grid-cols-1 lg:grid-cols-12 gap-6 items-start">
          
          {/* Left Column: Input Controls (Sticky on Desktop) */}
          <div className="lg:col-span-3 lg:sticky lg:top-6 space-y-6">
             <div className="glass-panel rounded-3xl p-6">
               <ControlPanel 
                 initialParams={params} 
                 onCalculate={handleCalculate} 
                 isLoading={loading} 
               />
             </div>
             
             {/* Error Message Block */}
             {error && (
              <div className="glass-panel p-4 border-red-500/30 bg-red-950/20 rounded-2xl text-red-200 text-xs leading-relaxed">
                <strong className="text-red-400">System Error:</strong><br/>{error}
              </div>
             )}
          </div>

          {/* Right Column: Results */}
          <div className="lg:col-span-9 grid grid-cols-1 gap-6">
            
            {/* Timeline Navigation */}
            <div className="glass-panel rounded-3xl p-1 overflow-hidden">
              <Timeline 
                  currentYear={new Date(params.datetime).getFullYear()} 
                  currentDatetime={params.datetime}
                  onYearChange={handleTimelineYearChange}
                  mapping={data?.fortune?.mapping_record}
              />
            </div>

            {/* Skeleton State */}
            {loading && !data && (
              <div className="space-y-6">
                 <div className="h-[400px] glass-panel rounded-3xl flex items-center justify-center text-gray-500 font-light tracking-widest animate-pulse">
                    CALCULATING CELESTIAL POSITIONS...
                 </div>
              </div>
            )}

            {/* Data Display */}
            {data && (
              <>
                {/* Top Row: Sky Map & Quick Info */}
                <div className="grid grid-cols-1 xl:grid-cols-2 gap-6">
                   <div className="h-[500px] xl:h-[600px]">
                          <SkyCard 
                             data={data.sky} 
                             date={new Date(params.datetime)}
                             lat={params.lat}
                             lon={params.lon}
                             timezone={timezoneOffset}
                             containerId="celestial-map"
                          />
                   </div>
                   
                   {/* Fortune Card - Major Display */}
                 <div>
                     <FortuneCard 
                       data={data.fortune} 
                       currentYear={new Date(params.datetime).getFullYear()}
                       onJumpToYear={handleJumpToYear}
                     />
                  </div>
                   {compareMode && compareData && (
                     <>
                       <div className="h-[500px] xl:h-[600px]">
                          <SkyCard 
                             data={compareData.sky} 
                             date={new Date(compareDatetime || params.datetime)}
                             lat={params.lat}
                             lon={params.lon}
                             timezone={compareTimezoneOffset ?? timezoneOffset}
                             containerId="celestial-map-compare"
                          />
                       </div>
                       <div>
                          <FortuneCard 
                            data={compareData.fortune} 
                            currentYear={new Date(compareDatetime || params.datetime).getFullYear()}
                          />
                       </div>
                     </>
                   )}
                </div>
                {compareMode && compareData && (
                  <div className="mt-2 p-4 glass-panel rounded-2xl text-xs text-gray-400 font-mono flex justify-between">
                    <span>对比年卦: {data.fortune.hexagram_major} → {compareData.fortune.hexagram_major}</span>
                    <span>运: {data.fortune.yun} → {compareData.fortune.yun}</span>
                    <span>世: {data.fortune.shi} → {compareData.fortune.shi}</span>
                    <span>旬: {data.fortune.xun} → {compareData.fortune.xun}</span>
                  </div>
                )}
              </>
            )}
            
            {!data && !loading && !error && (
               <div className="h-[400px] glass-panel rounded-3xl flex flex-col items-center justify-center text-gray-500 border-2 border-dashed border-white/5">
                  <p className="text-xl font-serif tracking-widest mb-3 text-gold/50">等待指令</p>
                  <p className="text-xs uppercase tracking-wider opacity-50">Ready for Calculation</p>
               </div>
            )}

          </div>
        </div>
      </div>
      {debugOpen && <DebugPanel data={data} compare={compareData} />}
    </div>
  )
}

function DebugPanel({ data, compare }: { data: CombinedResponse | null; compare: CombinedResponse | null }) {
  if (!data) return null;
  const sky = data.sky;
  const fortune = data.fortune;
  return (
    <div className="fixed bottom-4 right-4 z-50 w-[360px] max-h-[70vh] overflow-auto bg-black/80 backdrop-blur-xl border border-white/10 rounded-2xl p-6 shadow-2xl">
      <div className="text-xs text-gold/50 uppercase tracking-widest mb-4 border-b border-white/5 pb-2">Debug Console</div>
      <div className="text-[10px] text-gray-400 font-mono space-y-3">
        <div>JD: {sky.jd?.toFixed(4) ?? 'N/A'} · GMST: {sky.gmst_deg?.toFixed(2) ?? 'N/A'} °</div>
        <div>ΔT: {sky.delta_t_sec?.toFixed(2) ?? 'N/A'} s</div>
        <div className="p-2 bg-white/5 rounded">Lat: {new URLSearchParams(window.location.search).get('lat') || ''} · Lon: {new URLSearchParams(window.location.search).get('lon') || ''}</div>
        
        <div className="mt-4 text-gray-300 font-bold">Celestial Bodies</div>
        <div className="space-y-1 max-h-[150px] overflow-y-auto custom-scrollbar">
          {sky.bodies.map(b => (
            <div key={b.name} className="flex justify-between border-b border-white/5 pb-1">
              <span className="text-cyan-300">{b.name}</span>
              <span>Alt {b.alt_deg.toFixed(1)}°</span>
            </div>
          ))}
        </div>

        <div className="mt-4 text-gray-300 font-bold">Mapping Data</div>
        <div className="p-2 bg-white/5 rounded">
          {fortune.mapping_record ? (
            <div className="space-y-1">
              <div className="text-gold">{fortune.mapping_record.gregorian_year} · {fortune.mapping_record.ganzhi}</div>
              <div>{fortune.mapping_record.dynasty}</div>
              <div className="text-gray-500">{fortune.mapping_record.person}</div>
            </div>
          ) : (
            <div className="italic opacity-50">No mapping record found</div>
          )}
        </div>
        
        {compare && (
          <div className="mt-4 text-gray-300 font-bold">Comparison Debug</div>
        )}
      </div>
    </div>
  );
}
