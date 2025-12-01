
import { useEffect, useRef, useState } from 'react';

declare global {
  interface Window {
    Celestial?: {
      display: (config: unknown) => void;
      skyview: (opts: { date: Date; location: [number, number]; timezone?: number }) => void;
      date: (d: Date) => void;
      location: (loc: [number, number]) => void;
    };
    __celestialTooltipCleanup?: () => void;
  }
}

interface StarMapProps {
  date: Date;
  lat: number;
  lon: number;
  _planets?: Array<{ name: string; alt_deg: number; az_deg: number }>;
  timezone?: number;
  containerId?: string;
}

export function StarMap({ date, lat, lon, timezone = 0, containerId = 'celestial-map' }: StarMapProps) {

  const initializedRef = useRef(false);
  const [showXiu, setShowXiu] = useState(true);
  const [renameZh, setRenameZh] = useState(true);
  const [showConst, setShowConst] = useState(true);
  const [culture, setCulture] = useState<'hj' | 'iau'>('hj');
  useEffect(() => {
    const originalAlert = window.alert;
    let retryHandle: number | null = null;
    const resolveDataRoot = async (): Promise<string> => {
      const API_BASE = (import.meta as unknown as { env?: Record<string, string> }).env?.VITE_BACKEND_URL || '';
      const required = [
        'stars.6.json','constellations.json','constellations.lines.json','constellations.bounds.json','mw.json','planets.json'
      ];
      const probe = async (root: string) => {
        try {
          for (const p of required) {
            const r = await fetch(`${root}${p}`, { method: 'GET' });
            const ct = r.headers.get('content-type') || '';
            if (!r.ok || !ct.includes('application/json')) return false;
            const txt = await r.text();
            if (!txt || txt.startsWith('404')) return false;
          }
          return true;
        } catch { return false; }
      };
      if (await probe('/data/')) return '/data/';
      if (API_BASE && await probe(`${API_BASE}/api/celestial/data/`)) return `${API_BASE}/api/celestial/data/`;
      if (await probe('/api/celestial/data/')) return '/api/celestial/data/';
      const cdns = [
        'https://cdn.jsdelivr.net/gh/ofrohn/celestial@master/data/',
        'https://fastly.jsdelivr.net/gh/ofrohn/celestial@master/data/',
        'https://raw.githubusercontent.com/ofrohn/celestial/master/data/',
        'https://ofrohn.github.io/data/'
      ];
      for (const root of cdns) { if (await probe(root)) return root; }
      return '/api/celestial/data/';
    };
    // removed unused resolveCulture
    const configBase = {
      width: 0, // Full width
      projection: "airy", 
      transform: "equatorial", 
      center: null, // [0,0,0]
      adaptable: true,
      interactive: true,
      form: false, // Disable built-in form
      location: false, // Disable built-in geolocation to prevent TimeZoneDB calls
      controls: false,
      lang: "zh",
      container: containerId,
      datapath: "/api/celestial/data/",
      stars: {
        show: true,
        limit: 6,
        colors: true,
        style: { fill: "#ffffff", opacity: 1 },
        data: 'stars.6.json',
        size: 7,
        exponent: -0.28,
        names: false,
        propername: true,
        propernamelimit: 2.5,
        namestyle: { fill: "#f6e58d", font: "bold 11px 'Lucida Sans Unicode', Trebuchet, Helvetica, Arial, sans-serif", align: "left", baseline: "top" },
        designation: false,
      },
      dsos: { show: false },
      constellations: {
        show: false,
        names: false,
        desig: false,
        lines: false,
        linestyle: { stroke: "#cccccc", width: 1.5, opacity: 0.7 },
        namesType: "name",
      },
      mw: { show: true, style: { fill: "#ffffff", opacity: 0.1 } },
      lines: {
        graticule: { show: true, stroke: "#cccccc", width: 0.6, opacity: 0.3 },
        equatorial: { show: true, stroke: "#aaaaaa", width: 1.3, opacity: 0.4 },
        ecliptic: { show: true, stroke: "#66cc66", width: 1.3, opacity: 0.6 },
        galactic: { show: false, stroke: "#cc6666", width: 1.3, opacity: 0.7 },
        supergalactic: { show: false, stroke: "#cc66cc", width: 1.3, opacity: 0.7 },
      },
      planets: {
        show: true,
        which: ["Sun", "Moon", "Mercury", "Venus", "Mars", "Jupiter", "Saturn"],
        symbols: {
          sun: { scale: 1.8, fill: "#ffff00", stroke: "#cccc00" },
          moon: { scale: 1.8, fill: "#eeeeee", stroke: "#cccccc" },
          planet: { scale: 1.4, fill: "#00f3ff", stroke: "#0099aa" },
        },
        names: true
      },
      horizon: {
        show: true,
        stroke: "#444444",
        width: 1.5,
        fill: "#000000",
        opacity: 0
      },
      daylight: { show: false }, // Show night sky for clarity
    };

    const boot = async () => {
      if (!window.Celestial) {
        retryHandle = window.setTimeout(boot, 200);
        return;
      }
      const API_BASE = (import.meta as unknown as { env?: Record<string, string> }).env?.VITE_BACKEND_URL || '';
      try {
        const r = await fetch(`${API_BASE || ''}/api/settings/sky`, { method: 'GET' });
        if (r.ok) {
          const s = await r.json() as { show_const: boolean; show_xiu: boolean; zh_planet_names: boolean; culture: string };
          setShowConst(!!s.show_const);
          setShowXiu(!!s.show_xiu);
          setRenameZh(!!s.zh_planet_names);
          setCulture((s.culture === 'hj' ? 'hj' : 'iau'));
        }
      } catch {}
      window.alert = (msg?: unknown) => {
        const s = String(msg || '');
        if (s.includes('Data file could not be loaded')) {
          console.warn('[Celestial]', s);
        } else {
          try { originalAlert(msg as unknown as string); } catch { void 0; }
        }
      };
      await new Promise<void>(resolve => setTimeout(resolve, 50));
      const root = await resolveDataRoot();
      const selector = `#${containerId}`;
      const el = document.querySelector(selector) as HTMLElement | null;
      const w = Math.max(1, Math.floor((el?.clientWidth || el?.getBoundingClientRect().width || (typeof window !== 'undefined' ? window.innerWidth : 800))));
      const containerEl = document.getElementById(containerId);
      if (containerEl) { containerEl.innerHTML = ''; initializedRef.current = false; }
      const config = { ...configBase, culture, datapath: root, container: containerId, width: w, constellations: { ...configBase.constellations, show: showConst, names: showConst, lines: showConst, namesType: 'name' } } as unknown;
      if (!initializedRef.current) {
        try { 
          window.Celestial!.display(config);
          initializedRef.current = true;
        } catch { /* noop */ }
      }
      const tzMinutes = Number.isFinite(timezone) ? Math.round(timezone * 60) : 0;
      const safeDate = date instanceof Date && !isNaN(date.getTime()) ? date : new Date();
      try {
        if (window.Celestial && typeof window.Celestial.skyview === 'function') {
          window.Celestial.skyview({ date: safeDate, location: [lat, lon], timezone: tzMinutes });
        } else {
          if (window.Celestial && typeof window.Celestial.date === 'function') window.Celestial.date(safeDate);
          if (window.Celestial && typeof window.Celestial.location === 'function') window.Celestial.location([lat, lon]);
        }
      } catch { /* noop */ }

      const ensureRendered = async () => {
        const c = document.querySelector(selector) as HTMLElement | null;
        const has = !!c && (!!c.querySelector('canvas') || !!c.querySelector('svg'));
        if (has) return;
      const fallback = { ...configBase, culture, datapath: root, container: containerId, width: w, projection: 'aitoff', constellations: { ...configBase.constellations, show: showConst, names: showConst, lines: showConst, namesType: 'name' } } as unknown;
        try { 
          window.Celestial!.display(fallback);
          initializedRef.current = true;
        } catch { /* noop */ }
        try {
          if (window.Celestial && typeof window.Celestial.skyview === 'function') {
            window.Celestial.skyview({ date: safeDate, location: [lat, lon], timezone: tzMinutes });
          }
        } catch { /* noop */ }
      };
      setTimeout(ensureRendered, 600);
    };
    boot();

    const container = document.getElementById(containerId);
    if (container) {
      let tooltip = container.querySelector('#celestial-tooltip') as HTMLDivElement | null;
      if (!tooltip) {
        tooltip = document.createElement('div') as HTMLDivElement;
        tooltip.id = 'celestial-tooltip';
        tooltip.style.position = 'absolute';
        tooltip.style.display = 'none';
        tooltip.style.padding = '6px 8px';
        tooltip.style.background = 'rgba(0,0,0,0.7)';
        tooltip.style.color = '#b5e6a0';
        tooltip.style.fontSize = '10px';
        tooltip.style.border = '1px solid rgba(102,204,102,0.5)';
        tooltip.style.borderRadius = '4px';
        tooltip.style.pointerEvents = 'none';
        container.appendChild(tooltip);
      }

      let cleanup: (() => void) | null = null;
      setTimeout(() => {
        const svg = container.querySelector('svg');
        const paths = Array.from((svg?.querySelectorAll('path') || []) as NodeListOf<SVGPathElement>);
        const eclPath = paths.find(p => getComputedStyle(p).stroke === 'rgb(102, 204, 102)');
        if (!eclPath) return;
        eclPath.style.pointerEvents = 'stroke';
        const text = '黄道（Ecliptic）：太阳的视运动路径，行星多沿此附近运行';
        const onEnter = (e: MouseEvent) => {
          const rect = container.getBoundingClientRect();
          tooltip.textContent = text;
          tooltip.style.left = `${e.clientX - rect.left + 8}px`;
          tooltip.style.top = `${e.clientY - rect.top + 8}px`;
          tooltip.style.display = 'block';
        };
        const onMove = (e: MouseEvent) => {
          const rect = container.getBoundingClientRect();
          tooltip.style.left = `${e.clientX - rect.left + 8}px`;
          tooltip.style.top = `${e.clientY - rect.top + 8}px`;
        };
        const onLeave = () => { if (tooltip) tooltip.style.display = 'none'; };
        eclPath.addEventListener('mouseenter', onEnter);
        eclPath.addEventListener('mousemove', onMove);
        eclPath.addEventListener('mouseleave', onLeave);

        const renamePlanets = () => {
          const texts = Array.from((svg?.querySelectorAll('text') || []) as NodeListOf<SVGTextElement>);
          const map: Record<string, string> = {
            'Sun': '太阳', 'Moon': '月亮', 'Mercury': '水星', 'Venus': '金星', 'Mars': '火星', 'Jupiter': '木星', 'Saturn': '土星'
          };
          texts.forEach(t => { const zh = map[t.textContent || '']; if (zh) t.textContent = zh; });
        };
        if (renameZh) renamePlanets();

        const mansions = ['角','亢','氐','房','心','尾','箕','斗','牛','女','虚','危','室','壁','奎','娄','胃','昴','毕','觜','参','井','鬼','柳','星','张','翼','轸'];
        const svgRect = svg!.getBoundingClientRect();
        const contRect = container.getBoundingClientRect();
        if (showXiu) {
          let overlay = container.querySelector('#xiu-overlay') as HTMLDivElement | null;
          if (overlay) overlay.remove();
          overlay = document.createElement('div');
          overlay.id = 'xiu-overlay';
          overlay.style.position = 'absolute';
          overlay.style.left = '0';
          overlay.style.top = '0';
          overlay.style.width = '100%';
          overlay.style.height = '100%';
          overlay.style.pointerEvents = 'none';
          container.appendChild(overlay);
          const total = eclPath.getTotalLength();
          const seg = total / mansions.length;
          for (let i = 0; i < mansions.length; i++) {
            const centerPt = eclPath.getPointAtLength((i + 0.5) * seg);
            const tickPt = eclPath.getPointAtLength(i * seg);
            const nextPt = eclPath.getPointAtLength(Math.min(total, i * seg + 1));
            const angleRad = Math.atan2(nextPt.y - tickPt.y, nextPt.x - tickPt.x);
            const angle = angleRad * 180 / Math.PI;
            const nx = -Math.sin(angleRad);
            const ny = Math.cos(angleRad);
            const off = 10;
            const label = document.createElement('div');
            label.textContent = mansions[i];
            label.style.position = 'absolute';
            label.style.left = `${svgRect.left - contRect.left + centerPt.x + nx * off}px`;
            label.style.top = `${svgRect.top - contRect.top + centerPt.y + ny * off}px`;
            label.style.transform = `translate(-50%, -50%) rotate(${angle}deg)`;
            label.style.color = '#f6e58d';
            label.style.fontSize = '10px';
            label.style.background = 'rgba(0,0,0,0.35)';
            label.style.padding = '2px 4px';
            label.style.borderRadius = '3px';
            overlay.appendChild(label);
            const tick = document.createElement('div');
            tick.style.position = 'absolute';
            tick.style.left = `${svgRect.left - contRect.left + tickPt.x}px`;
            tick.style.top = `${svgRect.top - contRect.top + tickPt.y}`;
            tick.style.width = '14px';
            tick.style.height = '1px';
            tick.style.background = '#f6e58d';
            tick.style.transform = `translate(-50%, -50%) rotate(${angle}deg)`;
            overlay.appendChild(tick);
          }
        }
        cleanup = () => {
          eclPath.removeEventListener('mouseenter', onEnter);
          eclPath.removeEventListener('mousemove', onMove);
          eclPath.removeEventListener('mouseleave', onLeave);
          if (tooltip) tooltip.remove();
          const old = container.querySelector('#xiu-overlay');
          if (old) old.remove();
        };
      }, 500);
      if (!initializedRef.current) {
        const restore = () => { if (cleanup) cleanup(); };
        window.__celestialTooltipCleanup = restore;
      }
    }

    return () => {
      if (retryHandle) { try { clearTimeout(retryHandle); } catch { void 0; } }
      const fn = window.__celestialTooltipCleanup;
      if (typeof fn === 'function') fn();
      try { window.alert = originalAlert; } catch { void 0; }
    };
  }, [date, lat, lon, timezone, containerId, showXiu, renameZh, showConst, culture]);

  

  return (
    <div style={{ width: '100%', height: '100%', position: 'relative', background: '#000' }}>
      <div id={containerId}></div>
      <div style={{ position: 'absolute', right: 8, top: 8, display: 'flex', gap: 8, background: 'rgba(0,0,0,0.4)', padding: '6px 8px', borderRadius: 6 }}>
        <label style={{ color: '#f6e58d', fontSize: 11 }}>
          <input type="checkbox" checked={showConst} onChange={async e => { const v = e.target.checked; setShowConst(v); const API_BASE = (import.meta as unknown as { env?: Record<string, string> }).env?.VITE_BACKEND_URL || ''; try { await fetch(`${API_BASE || ''}/api/settings/sky`, { method: 'POST', headers: { 'Content-Type': 'application/json' }, body: JSON.stringify({ show_const: v, show_xiu: showXiu, zh_planet_names: renameZh, culture }) }); } catch {} }} /> 星官
        </label>
        <label style={{ color: '#f6e58d', fontSize: 11 }}>
          <input type="checkbox" checked={showXiu} onChange={async e => { const v = e.target.checked; setShowXiu(v); const API_BASE = (import.meta as unknown as { env?: Record<string, string> }).env?.VITE_BACKEND_URL || ''; try { await fetch(`${API_BASE || ''}/api/settings/sky`, { method: 'POST', headers: { 'Content-Type': 'application/json' }, body: JSON.stringify({ show_const: showConst, show_xiu: v, zh_planet_names: renameZh, culture }) }); } catch {} }} /> 二十八宿
        </label>
        <label style={{ color: '#f6e58d', fontSize: 11 }}>
          <input type="checkbox" checked={renameZh} onChange={async e => { const v = e.target.checked; setRenameZh(v); const API_BASE = (import.meta as unknown as { env?: Record<string, string> }).env?.VITE_BACKEND_URL || ''; try { await fetch(`${API_BASE || ''}/api/settings/sky`, { method: 'POST', headers: { 'Content-Type': 'application/json' }, body: JSON.stringify({ show_const: showConst, show_xiu: showXiu, zh_planet_names: v, culture }) }); } catch {} }} /> 行星中文
        </label>
      </div>
      {/* Hidden stub form to satisfy d3-celestial internal updates (render once) */}
      {containerId === 'celestial-map' && (
        <div id="celestial-form" style={{ display: 'none' }}>
          <input type="text" id="datetime" defaultValue={new Date().toISOString()} />
        </div>
      )}
    </div>
  );
}
