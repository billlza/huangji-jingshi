
import { useEffect, useRef, useState } from 'react';

declare global {
  interface Window {
    Celestial?: {
      display: (config: unknown) => void;
      skyview: (opts: { date: Date; location: [number, number]; timezone?: number }) => void;
      date: (d: Date) => void;
      location: (loc: [number, number]) => void;
      add?: (data: unknown, opts: Record<string, unknown>) => void;
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
  const rootRef = useRef<string | null>(null);
  const settingsControllerRef = useRef<AbortController | null>(null);
  const [showXiu, setShowXiu] = useState(true);
  const [renameZh, setRenameZh] = useState(true);
  const [showConst, setShowConst] = useState(true);
  const [culture, setCulture] = useState<'hj' | 'cn' | 'iau'>('hj');
  const [onlyHj, setOnlyHj] = useState(false);
  const [playing, setPlaying] = useState(false);
  const [playSpeed, setPlaySpeed] = useState(60); // 时间流速倍数：1=实时, 60=1分钟/秒, 3600=1小时/秒
  const playTimerRef = useRef<number | null>(null);
  const playStartRef = useRef<{ realTime: number; simTime: Date } | null>(null);
  const [searchTerm, setSearchTerm] = useState('');
  const [gpsLocation, setGpsLocation] = useState<{ lat: number; lon: number } | null>(null);
  const [gpsLoading, setGpsLoading] = useState(false);
  
  // 获取 GPS 定位
  const requestGpsLocation = () => {
    if (!navigator.geolocation) {
      alert('您的浏览器不支持 GPS 定位');
      return;
    }
    setGpsLoading(true);
    navigator.geolocation.getCurrentPosition(
      (pos) => {
        setGpsLocation({ lat: pos.coords.latitude, lon: pos.coords.longitude });
        setGpsLoading(false);
        console.log(`[StarMap] GPS location: ${pos.coords.latitude}, ${pos.coords.longitude}`);
      },
      (err) => {
        console.warn('[StarMap] GPS error:', err.message);
        setGpsLoading(false);
        alert('无法获取 GPS 定位: ' + err.message);
      },
      { enableHighAccuracy: true, timeout: 10000 }
    );
  };
  
  // 实际使用的坐标（优先 GPS）
  const actualLat = gpsLocation?.lat ?? lat;
  const actualLon = gpsLocation?.lon ?? lon;
  const USE_REMOTE_SETTINGS = false;
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
            if (!r.ok) return false;
            const txt = await r.text();
            if (!txt || txt.startsWith('404')) return false;
            let obj: unknown;
            try { obj = JSON.parse(txt); } catch { return false; }
            if (typeof obj !== 'object' || obj === null) return false;
          }
          return true;
        } catch { return false; }
      };
      // 优先使用前端本地静态文件（/data/ 对应 public/data/）
      if (await probe('/data/')) {
        console.log('[StarMap] Using local static: /data/');
        return '/data/';
      }
      // 然后尝试后端 API
      if (await probe('/api/celestial/data/')) {
        console.log('[StarMap] Using local API: /api/celestial/data/');
        return '/api/celestial/data/';
      }
      if (API_BASE && await probe(`${API_BASE}/api/celestial/data/`)) {
        console.log(`[StarMap] Using API_BASE: ${API_BASE}/api/celestial/data/`);
        return `${API_BASE}/api/celestial/data/`;
      }
      console.warn('[StarMap] Local sources failed, falling back to CDN');
      const cdns = [
        'https://cdn.jsdelivr.net/gh/ofrohn/celestial@master/data/',
        'https://fastly.jsdelivr.net/gh/ofrohn/celestial@master/data/',
        'https://raw.githubusercontent.com/ofrohn/celestial/master/data/',
        'https://ofrohn.github.io/celestial/data/'
      ];
      for (const root of cdns) { if (await probe(root)) return root; }
      return '/data/';
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
      datapath: "/data/",
      stars: {
        show: true,
        limit: 6,
        colors: true,
        style: { fill: "#ffffff", opacity: 1 },
        data: 'stars.6.json',
        size: 4,
        exponent: -0.28,
        names: false,
        propername: true,
        propernamelimit: 2.5,
        namestyle: { fill: "#f6e58d", font: "bold 10px 'Lucida Sans Unicode', Trebuchet, Helvetica, Arial, sans-serif", align: "left", baseline: "top" },
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
      const SUPABASE_ANON_KEY = (import.meta as unknown as { env?: Record<string, string> }).env?.VITE_SUPABASE_ANON_KEY || '';
      try {
        if (API_BASE && USE_REMOTE_SETTINGS) {
          if (settingsControllerRef.current) { try { settingsControllerRef.current.abort(); } catch { /* ignore */ } }
          settingsControllerRef.current = new AbortController();
          const r = await fetch(`${API_BASE}/sky`, {
            method: 'GET',
            headers: {
              'Authorization': `Bearer ${SUPABASE_ANON_KEY}`,
              'Content-Type': 'application/json'
            },
            signal: settingsControllerRef.current.signal
          });
          if (r.ok) {
            const s = await r.json() as { show_const: boolean; show_xiu: boolean; zh_planet_names: boolean; culture: string };
            setShowConst(!!s.show_const);
            setShowXiu(!!s.show_xiu);
            setRenameZh(!!s.zh_planet_names);
            setCulture(s.culture === 'hj' ? 'hj' : (s.culture === 'cn' ? 'cn' : 'iau'));
          }
        }
      } catch { void 0; }
      window.alert = (msg?: unknown) => {
        const s = String(msg || '');
        if (s.includes('Data file could not be loaded')) {
          console.warn('[Celestial]', s);
        } else {
          try { originalAlert(msg as unknown as string); } catch { void 0; }
        }
      };
      await new Promise<void>(resolve => setTimeout(resolve, 50));
      const root = rootRef.current ?? await resolveDataRoot();
      rootRef.current = root;
      console.log('[StarMap] Resolved data root:', root);
      const selector = `#${containerId}`;
      const el = document.querySelector(selector) as HTMLElement | null;
      const w = Math.max(1, Math.floor((el?.clientWidth || el?.getBoundingClientRect().width || (typeof window !== 'undefined' ? window.innerWidth : 800))));
      // 不再在每次 effect 执行时清空容器，避免在开发环境 StrictMode 下出现闪烁/消失
      // 仅在首次初始化时调用 display，其后只更新日期/位置
      const isIau = culture === 'iau';
      const config = { ...configBase, culture, datapath: root, container: containerId, width: w, constellations: { ...configBase.constellations, show: isIau ? showConst : false, names: isIau ? showConst : false, lines: isIau ? showConst : false, boundaries: isIau ? showXiu : false, namesType: isIau ? 'name' : 'name' } } as unknown;
      console.log('[StarMap] Celestial config:', { datapath: root, width: w, stars: (config as any).stars });
      // @ts-ignore - 保留以备将来使用
      const _drawCn = async (base: string) => {
        try {
          const [pts, lines, bounds] = await Promise.all([
            fetch(`${base}constellations.cn.json`).then(r => r.ok ? r.json() : null),
            fetch(`${base}constellations.lines.cn.json`).then(r => r.ok ? r.json() : null),
            fetch(`${base}constellations.bounds.cn.json`).then(r => r.ok ? r.json() : null),
          ]);
          if (pts && window.Celestial && typeof window.Celestial.add === 'function') {
            window.Celestial.add({ type: 'raw', callback: () => {}, redraw: () => {} }, pts);
          }
          if (lines && window.Celestial && typeof window.Celestial.add === 'function' && showConst) {
            window.Celestial.add({ type: 'raw', callback: () => {}, redraw: () => {} }, lines);
          }
          if (bounds && window.Celestial && typeof window.Celestial.add === 'function' && showXiu) {
            window.Celestial.add({ type: 'raw', callback: () => {}, redraw: () => {} }, bounds);
          }
        } catch { void 0; }
      };
      // @ts-ignore - 保留以备将来使用
      const _drawStars = async (base: string) => {
        try {
          const stars = await fetch(`${base}stars.6.json`).then(r => r.ok ? r.json() : null);
          if (stars && window.Celestial && typeof window.Celestial.add === 'function') {
            window.Celestial.add(stars, { type: 'point', style: { fill: '#ffffff', opacity: 0.9 }, size: 1.6 });
          }
        } catch { void 0; }
      };
      const markHjGroups = () => {
        const el = document.getElementById(containerId);
        const svg = el?.querySelector('svg');
        if (!svg) return;
        const groups = Array.from(svg.querySelectorAll('g')) as SVGElement[];
        groups.forEach(g => {
          const cls = (g.getAttribute('class') || '').toLowerCase();
          if (cls.includes('hj-star') || cls.includes('xiu-point')) {
            g.setAttribute('data-hj', '1');
          }
        });
      };
      // @ts-ignore - 保留以备将来使用
      const _drawHjStars = async (base: string) => {
        try {
          const hj = await fetch(`${base}cultures/huangji-stars.json`).then(r => r.ok ? r.json() : null);
          if (!hj || !window.Celestial || typeof window.Celestial.add !== 'function') return;
          
          // hip fallback: fill missing coordinates using stars.6.json
          let resolved = hj;
          try {
            const stars = await fetch(`${base}stars.6.json`).then(r => r.ok ? r.json() : null);
            const map = new Map<number, [number, number]>();
            if (stars && Array.isArray(stars.features)) {
              for (const f of stars.features as Array<{ id?: number; geometry?: { coordinates?: [number, number] } }>) {
                if (typeof f.id === 'number' && Array.isArray(f.geometry?.coordinates)) {
                  map.set(f.id, f.geometry!.coordinates as [number, number]);
                }
              }
            }
            if (Array.isArray(hj.features)) {
              const feats = (hj.features as Array<any>).map(feat => {
                const hip = typeof feat.properties?.hip === 'number' ? feat.properties.hip : Number(feat.properties?.hip);
                const coords = feat.geometry?.coordinates;
                if ((!Array.isArray(coords) || coords.length !== 2 || coords.some(n => typeof n !== 'number')) && Number.isFinite(hip) && map.has(hip)) {
                  feat.geometry = feat.geometry || { type: 'Point', coordinates: [0, 0] };
                  feat.geometry.coordinates = map.get(hip)!;
                }
                return feat;
              });
              resolved = { type: 'FeatureCollection', features: feats };
            }
          } catch { /* ignore hip fallback errors */ }
          
          // Store resolved features for later label rendering
          (window as unknown as { __hjStarsData?: unknown }).__hjStarsData = resolved;
          
          window.Celestial.add(resolved, {
            type: 'point',
            class: 'hj-star',
            style: { fill: '#ffd66b', opacity: 1 },
            size: (d: { properties?: { mag?: number } }) => {
              const mag = d.properties?.mag ?? 4;
              return Math.max(2, Math.pow(6 - mag, 0.5) * 1.2);
            },
            labels: true,
            label: (d: { properties?: { name_hj?: string; name_cn?: string } }) => d.properties?.name_hj || d.properties?.name_cn || '',
            labelStyle: {
              fill: '#ffeaa7',
              font: "bold 11px '-apple-system','SF Pro Text','Segoe UI', Roboto, sans-serif",
              align: 'left',
              baseline: 'middle'
            }
          });
          setTimeout(markHjGroups, 0);
          
          // Manual label rendering - find circles and add text labels next to them
          setTimeout(() => {
            const el = document.getElementById(containerId);
            const svg = el?.querySelector('svg');
            if (!svg) return;
            
            // Remove existing hj-labels
            svg.querySelectorAll('.hj-label').forEach(l => l.remove());
            
            const ns = 'http://www.w3.org/2000/svg';
            
            // Create a group for labels
            let labelGroup = svg.querySelector('.hj-labels-group') as SVGGElement | null;
            if (!labelGroup) {
              labelGroup = document.createElementNS(ns, 'g') as SVGGElement;
              labelGroup.setAttribute('class', 'hj-labels-group');
              svg.appendChild(labelGroup);
            } else {
              labelGroup.innerHTML = '';
            }
            
            // Try to find hj-star elements with bound data
            const hjStarGroups = Array.from(svg.querySelectorAll('g.hj-star')) as SVGGElement[];
            type DataBound = { __data__?: { properties?: { name_hj?: string; name_cn?: string } } };
            
            if (hjStarGroups.length > 0) {
              hjStarGroups.forEach((g) => {
                const circle = g.querySelector('circle');
                if (!circle) return;
                const cx = parseFloat(circle.getAttribute('cx') || '0');
                const cy = parseFloat(circle.getAttribute('cy') || '0');
                
                // Get name from bound data
                const data = (g as unknown as DataBound).__data__ || (circle as unknown as DataBound).__data__;
                const name = data?.properties?.name_hj || data?.properties?.name_cn || '';
                if (!name) return;
                
                const text = document.createElementNS(ns, 'text') as SVGTextElement;
                text.setAttribute('class', 'hj-label');
                text.setAttribute('x', String(cx + 6));
                text.setAttribute('y', String(cy + 3));
                text.setAttribute('fill', '#ffeaa7');
                text.setAttribute('font-size', '10px');
                text.setAttribute('font-weight', 'bold');
                text.setAttribute('font-family', "-apple-system, 'SF Pro Text', 'Segoe UI', Roboto, sans-serif");
                text.setAttribute('pointer-events', 'none');
                text.style.textShadow = '1px 1px 2px rgba(0,0,0,0.9), -1px -1px 2px rgba(0,0,0,0.9)';
                text.textContent = name;
                labelGroup!.appendChild(text);
              });
            }
            
            // Fallback: find all circles with gold fill and bound data
            if (labelGroup!.children.length === 0) {
              const allCircles = Array.from(svg.querySelectorAll('circle')) as SVGCircleElement[];
              allCircles.forEach((circle) => {
                const fill = circle.getAttribute('fill') || getComputedStyle(circle).fill;
                const isGold = fill === '#ffd66b' || fill === 'rgb(255, 214, 107)';
                if (!isGold) return;
                
                const cx = parseFloat(circle.getAttribute('cx') || '0');
                const cy = parseFloat(circle.getAttribute('cy') || '0');
                
                // Get name from bound data
                const data = (circle as unknown as DataBound).__data__ || (circle.parentElement as unknown as DataBound)?.__data__;
                const name = data?.properties?.name_hj || data?.properties?.name_cn || '';
                if (!name) return;
                
                const text = document.createElementNS(ns, 'text') as SVGTextElement;
                text.setAttribute('class', 'hj-label');
                text.setAttribute('x', String(cx + 6));
                text.setAttribute('y', String(cy + 3));
                text.setAttribute('fill', '#ffeaa7');
                text.setAttribute('font-size', '10px');
                text.setAttribute('font-weight', 'bold');
                text.setAttribute('font-family', "-apple-system, 'SF Pro Text', 'Segoe UI', Roboto, sans-serif");
                text.setAttribute('pointer-events', 'none');
                text.style.textShadow = '1px 1px 2px rgba(0,0,0,0.9), -1px -1px 2px rgba(0,0,0,0.9)';
                text.textContent = name;
                labelGroup!.appendChild(text);
              });
            }
          }, 1500);
        } catch { void 0; }
      };
      // @ts-ignore - 保留以备将来使用
      const _drawXiuPrincipal = async () => {
        if (!window.Celestial || typeof window.Celestial.add !== 'function') return;
        const eps = 23.439281 * Math.PI / 180;
        const mansions: Array<{ key: string; name: string; idx: number }> = [
          { key: 'jiao', name: '角', idx: 1 },{ key: 'kang', name: '亢', idx: 2 },{ key: 'di', name: '氐', idx: 3 },{ key: 'fang', name: '房', idx: 4 },{ key: 'xin', name: '心', idx: 5 },{ key: 'wei', name: '尾', idx: 6 },{ key: 'ji', name: '箕', idx: 7 },
          { key: 'dou', name: '斗', idx: 8 },{ key: 'niu', name: '牛', idx: 9 },{ key: 'nv', name: '女', idx: 10 },{ key: 'xu', name: '虚', idx: 11 },{ key: 'wei2', name: '危', idx: 12 },{ key: 'shi', name: '室', idx: 13 },{ key: 'bi', name: '壁', idx: 14 },
          { key: 'kui', name: '奎', idx: 15 },{ key: 'lou', name: '娄', idx: 16 },{ key: 'wei3', name: '胃', idx: 17 },{ key: 'mao', name: '昴', idx: 18 },{ key: 'bi2', name: '毕', idx: 19 },{ key: 'zi', name: '觜', idx: 20 },{ key: 'shen', name: '参', idx: 21 },
          { key: 'jing', name: '井', idx: 22 },{ key: 'gui', name: '鬼', idx: 23 },{ key: 'liu', name: '柳', idx: 24 },{ key: 'xing', name: '星', idx: 25 },{ key: 'zhang', name: '张', idx: 26 },{ key: 'yi', name: '翼', idx: 27 },{ key: 'zhen', name: '轸', idx: 28 }
        ];
        const L0 = 0; const span = 360 / 28; // 12.857142...
        const feats = mansions.map(m => {
          const lam = (L0 + (m.idx - 0.5) * span) * Math.PI / 180; // midpoint
          const sinlam = Math.sin(lam), coslam = Math.cos(lam);
          const ra = Math.atan2(sinlam * Math.cos(eps), coslam) * 180 / Math.PI;
          const dec = Math.asin(Math.sin(eps) * sinlam) * 180 / Math.PI;
          const RA = ((ra % 360) + 360) % 360;
          return {
            type: 'Feature',
            id: `xiu_${m.key}`,
            properties: { name_cn: m.name, name_hj: m.name, mag: 2.5, note: '二十八宿中点标注' },
            geometry: { type: 'Point', coordinates: [RA, dec] }
          };
        });
        const fc = { type: 'FeatureCollection', features: feats } as unknown as Record<string, unknown>;
        
        // Store for later label rendering
        (window as unknown as { __xiuData?: unknown }).__xiuData = fc;
        
        window.Celestial.add(fc, {
          type: 'point',
          class: 'xiu-point',
          style: { fill: '#f6e58d', opacity: 1 },
          size: 2.5,
          labels: true,
          label: (d: { properties?: { name_hj?: string; name_cn?: string } }) => d.properties?.name_hj || d.properties?.name_cn || '',
          labelStyle: { fill: '#f6e58d', font: "bold 12px '-apple-system','SF Pro Text','Segoe UI', Roboto, sans-serif", align: 'left', baseline: 'middle' }
        } as unknown as Record<string, unknown>);
        setTimeout(markHjGroups, 0);
        
        // Manual label rendering for Xiu (28 Mansions)
        setTimeout(() => {
          const el = document.getElementById(containerId);
          const svg = el?.querySelector('svg');
          if (!svg) return;
          
          // Remove existing xiu-labels
          svg.querySelectorAll('.xiu-label').forEach(l => l.remove());
          
          const ns = 'http://www.w3.org/2000/svg';
          let labelGroup = svg.querySelector('.xiu-labels-group') as SVGGElement | null;
          if (!labelGroup) {
            labelGroup = document.createElementNS(ns, 'g') as SVGGElement;
            labelGroup.setAttribute('class', 'xiu-labels-group');
            svg.appendChild(labelGroup);
          } else {
            labelGroup.innerHTML = '';
          }
          
          type DataBound = { __data__?: { properties?: { name_hj?: string; name_cn?: string } } };
          
          // Find xiu-point circles
          const xiuGroups = Array.from(svg.querySelectorAll('g.xiu-point')) as SVGGElement[];
          
          if (xiuGroups.length > 0) {
            xiuGroups.forEach((g) => {
              const circle = g.querySelector('circle');
              if (!circle) return;
              const cx = parseFloat(circle.getAttribute('cx') || '0');
              const cy = parseFloat(circle.getAttribute('cy') || '0');
              
              // Get name from bound data
              const data = (g as unknown as DataBound).__data__ || (circle as unknown as DataBound).__data__;
              const name = data?.properties?.name_hj || data?.properties?.name_cn || '';
              if (!name) return;
              
              const text = document.createElementNS(ns, 'text') as SVGTextElement;
              text.setAttribute('class', 'xiu-label');
              text.setAttribute('x', String(cx + 5));
              text.setAttribute('y', String(cy + 3));
              text.setAttribute('fill', '#f6e58d');
              text.setAttribute('font-size', '11px');
              text.setAttribute('font-weight', 'bold');
              text.setAttribute('font-family', "-apple-system, 'SF Pro Text', 'Segoe UI', Roboto, sans-serif");
              text.setAttribute('pointer-events', 'none');
              text.style.textShadow = '1px 1px 2px rgba(0,0,0,0.9), -1px -1px 2px rgba(0,0,0,0.9)';
              text.textContent = name;
              labelGroup!.appendChild(text);
            });
          }
          
          // Fallback: look for circles with xiu yellow color and bound data
          if (labelGroup!.children.length === 0) {
            const allCircles = Array.from(svg.querySelectorAll('circle')) as SVGCircleElement[];
            allCircles.forEach((circle) => {
              const fill = circle.getAttribute('fill') || getComputedStyle(circle).fill;
              const isYellow = fill === '#f6e58d' || fill === 'rgb(246, 229, 141)';
              if (!isYellow) return;
              
              const cx = parseFloat(circle.getAttribute('cx') || '0');
              const cy = parseFloat(circle.getAttribute('cy') || '0');
              
              // Get name from bound data
              const data = (circle as unknown as DataBound).__data__ || (circle.parentElement as unknown as DataBound)?.__data__;
              const name = data?.properties?.name_hj || data?.properties?.name_cn || '';
              if (!name) return;
              
              const text = document.createElementNS(ns, 'text') as SVGTextElement;
              text.setAttribute('class', 'xiu-label');
              text.setAttribute('x', String(cx + 5));
              text.setAttribute('y', String(cy + 3));
              text.setAttribute('fill', '#f6e58d');
              text.setAttribute('font-size', '11px');
              text.setAttribute('font-weight', 'bold');
              text.setAttribute('font-family', "-apple-system, 'SF Pro Text', 'Segoe UI', Roboto, sans-serif");
              text.setAttribute('pointer-events', 'none');
              text.style.textShadow = '1px 1px 2px rgba(0,0,0,0.9), -1px -1px 2px rgba(0,0,0,0.9)';
              text.textContent = name;
              labelGroup!.appendChild(text);
            });
          }
        }, 1500);
      };
      // 使用 Celestial 的投影函数将赤经赤纬转换为 SVG 坐标
      const projectCoords = (ra: number, dec: number, svg: SVGSVGElement): [number, number] | null => {
        // 尝试使用 Celestial 的投影函数
        const Cel = window.Celestial as unknown as { 
          projection?: { 
            (coords: [number, number]): [number, number] | null 
          };
          mapProjection?: {
            (coords: [number, number]): [number, number] | null
          };
        };
        
        if (Cel?.projection) {
          const result = Cel.projection([ra, dec]);
          if (result && !isNaN(result[0]) && !isNaN(result[1])) {
            return result;
          }
        }
        if (Cel?.mapProjection) {
          const result = Cel.mapProjection([ra, dec]);
          if (result && !isNaN(result[0]) && !isNaN(result[1])) {
            return result;
          }
        }
        
        // 回退：使用 Airy 投影的近似实现
        const svgRect = svg.getBoundingClientRect();
        const width = svgRect.width || 800;
        const height = svgRect.height || 800;
        const cx = width / 2;
        const cy = height / 2;
        const scale = Math.min(width, height) / 2 * 0.9;
        
        // 转换为弧度
        const raRad = ra * Math.PI / 180;
        const decRad = dec * Math.PI / 180;
        
        // Airy 投影近似（对于赤道坐标系）
        const cosDec = Math.cos(decRad);
        const x = cx - scale * cosDec * Math.sin(raRad);
        const y = cy - scale * Math.sin(decRad);
        
        // 检查点是否在可见区域内
        if (x < 0 || x > width || y < 0 || y > height) {
          return null;
        }
        
        return [x, y];
      };
      
      // 直接在 SVG 上绘制皇极星标签
      const drawHjStarsDirect = async (base: string) => {
        try {
          const hj = await fetch(`${base}cultures/huangji-stars.json`).then(r => r.ok ? r.json() : null);
          if (!hj) {
            console.warn('[StarMap] Failed to load huangji-stars.json');
            return;
          }
          
          // 获取 stars.6.json 用于坐标填充
          const starsData = await fetch(`${base}stars.6.json`).then(r => r.ok ? r.json() : null);
          const hipMap = new Map<number, [number, number]>();
          if (starsData?.features) {
            for (const f of starsData.features) {
              if (typeof f.id === 'number' && f.geometry?.coordinates) {
                hipMap.set(f.id, f.geometry.coordinates);
              }
            }
          }
          
          const el = document.getElementById(containerId);
          const svg = el?.querySelector('svg');
          if (!svg) {
            console.warn('[StarMap] SVG not found');
            return;
          }
          
          // 清除旧标签
          svg.querySelectorAll('.hj-star-label, .hj-star-marker').forEach(l => l.remove());
          
          const ns = 'http://www.w3.org/2000/svg';
          let labelGroup = svg.querySelector('.hj-star-labels') as SVGGElement | null;
          if (!labelGroup) {
            labelGroup = document.createElementNS(ns, 'g');
            labelGroup.setAttribute('class', 'hj-star-labels');
            svg.appendChild(labelGroup);
          } else {
            labelGroup.innerHTML = '';
          }
          
          let count = 0;
          for (const feat of (hj.features || [])) {
            const props = feat.properties || {};
            const name = props.name_hj || props.name_cn || '';
            if (!name) continue;
            
            let coords = feat.geometry?.coordinates;
            // 如果没有坐标，尝试从 HIP 获取
            if ((!coords || coords.length !== 2) && props.hip) {
              const hip = Number(props.hip);
              if (hipMap.has(hip)) {
                coords = hipMap.get(hip);
              }
            }
            if (!coords || coords.length !== 2) continue;
            
            const [ra, dec] = coords;
            const projected = projectCoords(ra, dec, svg);
            if (!projected) continue;
            
            const [x, y] = projected;
            
            // 绘制星点标记
            const circle = document.createElementNS(ns, 'circle');
            circle.setAttribute('class', 'hj-star-marker');
            circle.setAttribute('cx', String(x));
            circle.setAttribute('cy', String(y));
            circle.setAttribute('r', '3');
            circle.setAttribute('fill', '#ffd66b');
            circle.setAttribute('stroke', '#ffaa00');
            circle.setAttribute('stroke-width', '1');
            labelGroup.appendChild(circle);
            
            // 绘制标签
            const text = document.createElementNS(ns, 'text');
            text.setAttribute('class', 'hj-star-label');
            text.setAttribute('x', String(x + 6));
            text.setAttribute('y', String(y + 3));
            text.setAttribute('fill', '#ffd66b');
            text.setAttribute('font-size', '10');
            text.setAttribute('font-weight', 'bold');
            text.setAttribute('font-family', "-apple-system, 'SF Pro Text', 'PingFang SC', sans-serif");
            text.style.textShadow = '1px 1px 2px #000, -1px -1px 2px #000, 0 0 4px #000';
            text.textContent = name;
            labelGroup.appendChild(text);
            count++;
          }
          console.log(`[StarMap] Drew ${count} Huangji star labels`);
        } catch (e) { console.warn('[StarMap] drawHjStarsDirect error:', e); }
      };
      
      // 直接绘制二十八宿标签（沿黄道分布）
      const drawXiuLabelsDirect = () => {
        const el = document.getElementById(containerId);
        const svg = el?.querySelector('svg');
        if (!svg) return;
        
        // 清除旧标签
        svg.querySelectorAll('.xiu-label-direct, .xiu-marker').forEach(l => l.remove());
        
        const ns = 'http://www.w3.org/2000/svg';
        let labelGroup = svg.querySelector('.xiu-labels-direct') as SVGGElement | null;
        if (!labelGroup) {
          labelGroup = document.createElementNS(ns, 'g');
          labelGroup.setAttribute('class', 'xiu-labels-direct');
          svg.appendChild(labelGroup);
        } else {
          labelGroup.innerHTML = '';
        }
        
        // 二十八宿名称和对应的黄经起点（传统）
        const mansions = [
          { name: '角', lon: 0 }, { name: '亢', lon: 12 }, { name: '氐', lon: 24 }, { name: '房', lon: 36 },
          { name: '心', lon: 48 }, { name: '尾', lon: 60 }, { name: '箕', lon: 72 }, { name: '斗', lon: 84 },
          { name: '牛', lon: 96 }, { name: '女', lon: 108 }, { name: '虚', lon: 120 }, { name: '危', lon: 132 },
          { name: '室', lon: 144 }, { name: '壁', lon: 156 }, { name: '奎', lon: 168 }, { name: '娄', lon: 180 },
          { name: '胃', lon: 192 }, { name: '昴', lon: 204 }, { name: '毕', lon: 216 }, { name: '觜', lon: 228 },
          { name: '参', lon: 240 }, { name: '井', lon: 252 }, { name: '鬼', lon: 264 }, { name: '柳', lon: 276 },
          { name: '星', lon: 288 }, { name: '张', lon: 300 }, { name: '翼', lon: 312 }, { name: '轸', lon: 324 }
        ];
        
        const eps = 23.439281 * Math.PI / 180; // 黄赤交角
        
        let count = 0;
        for (const m of mansions) {
          // 将黄经转换为赤经赤纬
          const lamRad = (m.lon + 6) * Math.PI / 180; // 取宿的中点
          const sinLam = Math.sin(lamRad);
          const cosLam = Math.cos(lamRad);
          
          // 黄道坐标到赤道坐标转换
          const ra = Math.atan2(sinLam * Math.cos(eps), cosLam) * 180 / Math.PI;
          const dec = Math.asin(Math.sin(eps) * sinLam) * 180 / Math.PI;
          const RA = ((ra % 360) + 360) % 360;
          
          const projected = projectCoords(RA, dec, svg);
          if (!projected) continue;
          
          const [x, y] = projected;
          
          // 绘制宿点标记
          const circle = document.createElementNS(ns, 'circle');
          circle.setAttribute('class', 'xiu-marker');
          circle.setAttribute('cx', String(x));
          circle.setAttribute('cy', String(y));
          circle.setAttribute('r', '4');
          circle.setAttribute('fill', '#f6e58d');
          circle.setAttribute('stroke', '#d4a855');
          circle.setAttribute('stroke-width', '1');
          labelGroup.appendChild(circle);
          
          // 绘制标签
          const text = document.createElementNS(ns, 'text');
          text.setAttribute('class', 'xiu-label-direct');
          text.setAttribute('x', String(x));
          text.setAttribute('y', String(y - 8));
          text.setAttribute('fill', '#f6e58d');
          text.setAttribute('font-size', '11');
          text.setAttribute('font-weight', 'bold');
          text.setAttribute('font-family', "-apple-system, 'SF Pro Text', 'PingFang SC', sans-serif");
          text.setAttribute('text-anchor', 'middle');
          text.style.textShadow = '1px 1px 2px #000, -1px -1px 2px #000, 0 0 4px #000';
          text.textContent = m.name;
          labelGroup.appendChild(text);
          count++;
        }
        console.log(`[StarMap] Drew ${count} Xiu (28 Mansions) labels`);
      };
      
      try { 
        window.Celestial!.display(config);
        initializedRef.current = true;
      } catch { /* noop */ }
      const tzMinutes = Number.isFinite(timezone) ? Math.round(timezone * 60) : 0;
      const safeDate = date instanceof Date && !isNaN(date.getTime()) ? date : new Date();
      try {
        if (window.Celestial && typeof window.Celestial.skyview === 'function') {
          window.Celestial.skyview({ date: safeDate, location: [actualLat, actualLon], timezone: tzMinutes });
        } else {
          if (window.Celestial && typeof window.Celestial.date === 'function') window.Celestial.date(safeDate);
          if (window.Celestial && typeof window.Celestial.location === 'function') window.Celestial.location([lat, lon]);
        }
      } catch { /* noop */ }

      const ensureRendered = async () => {
        const c = document.querySelector(selector) as HTMLElement | null;
        const canvas = c?.querySelector('canvas') as HTMLCanvasElement | null;
        const svg = c?.querySelector('svg') as SVGSVGElement | null;
        const vis = (el: Element | null) => !!el && el.getBoundingClientRect().width > 20 && el.getBoundingClientRect().height > 20;
        const hasVisible = vis(canvas) || vis(svg);
        if (hasVisible) return;
        const fallback = { ...configBase, culture, datapath: (rootRef.current ?? root), container: containerId, width: Math.max(720, w), projection: 'aitoff', constellations: { ...configBase.constellations, show: isIau ? showConst : false, names: isIau ? showConst : false, lines: isIau ? showConst : false, boundaries: isIau ? showXiu : false, namesType: isIau ? 'name' : 'name' } } as unknown;
        try { 
          window.Celestial!.display(fallback);
          initializedRef.current = true;
        } catch { /* noop */ }
        try {
          if (window.Celestial && typeof window.Celestial.skyview === 'function') {
            window.Celestial.skyview({ date: safeDate, location: [actualLat, actualLon], timezone: tzMinutes });
          }
        } catch { /* noop */ }

        const c2 = document.querySelector(selector) as HTMLElement | null;
        const canvas2 = c2?.querySelector('canvas') as HTMLCanvasElement | null;
        const svg2 = c2?.querySelector('svg') as SVGSVGElement | null;
        if (vis(canvas2) || vis(svg2)) return;
        // 强制回退到 CDN 根
        const hardRoot = 'https://cdn.jsdelivr.net/gh/ofrohn/celestial@master/data/';
        const hard = { ...configBase, culture, datapath: hardRoot, container: containerId, width: Math.max(720, w), projection: 'aitoff', constellations: { ...configBase.constellations, show: isIau ? showConst : false, names: isIau ? showConst : false, lines: isIau ? showConst : false, boundaries: isIau ? showXiu : false, namesType: isIau ? 'name' : 'name' } } as unknown;
        try {
          window.Celestial!.display(hard);
          initializedRef.current = true;
        } catch { /* noop */ }
        try {
          if (window.Celestial && typeof window.Celestial.skyview === 'function') {
            window.Celestial.skyview({ date: safeDate, location: [actualLat, actualLon], timezone: tzMinutes });
          }
        } catch { /* noop */ }
        const c3 = document.querySelector(selector) as HTMLElement | null;
        const canvas3 = c3?.querySelector('canvas') as HTMLCanvasElement | null;
        const svg3 = c3?.querySelector('svg') as SVGSVGElement | null;
        const ok3 = vis(canvas3) || vis(svg3);
        if (!ok3 && c3) {
          // 若已有元素但不可见，做一次强制重建
          try { c3.innerHTML = ''; } catch { /* ignore */ }
          try { window.Celestial!.display(hard); } catch { /* noop */ }
        }
      };
      setTimeout(async () => { 
        await ensureRendered(); 
        const base = rootRef.current ?? root; 
        console.log('[StarMap] Drawing overlays with base:', base);
        // 使用 SVG 直接绘制自定义标注，避免 Celestial.add() 的 redraw 问题
        if (!isIau) { 
          if (culture === 'hj') { 
            await drawHjStarsDirect(base); 
            if (showXiu) { 
              drawXiuLabelsDirect(); 
            } 
          } 
        } 
      }, 800);
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
      const applyOnlyHjVisibility = (only: boolean) => {
        const svg = container.querySelector('svg');
        if (!svg) return;
        const circles = Array.from((svg.querySelectorAll('circle') || []) as NodeListOf<SVGCircleElement>);
        circles.forEach(el => {
          const keep = !!el.closest('g[data-hj="1"]');
          el.style.opacity = only && !keep ? '0' : '';
        });
        const texts = Array.from((svg.querySelectorAll('text') || []) as NodeListOf<SVGTextElement>);
        texts.forEach(el => {
          const keep = !!el.closest('g[data-hj="1"]') || el.classList.contains('hj-star') || el.classList.contains('xiu-point');
          el.style.opacity = only && !keep ? '0' : '';
        });
      };
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
      setTimeout(() => {
        const svg = container.querySelector('svg');
        const points = Array.from((svg?.querySelectorAll('.hj-star') || []) as NodeListOf<SVGElement>);
        if (points.length === 0 || !tooltip) return;
        const ensureDetail = () => {
          let panel = container.querySelector('#hj-detail') as HTMLDivElement | null;
          if (!panel) {
            panel = document.createElement('div');
            panel.id = 'hj-detail';
            panel.style.position = 'absolute';
            panel.style.right = '8px';
            panel.style.bottom = '8px';
            panel.style.maxWidth = '320px';
            panel.style.background = 'rgba(0,0,0,0.55)';
            panel.style.border = '1px solid rgba(255,214,107,0.5)';
            panel.style.borderRadius = '6px';
            panel.style.padding = '8px';
            panel.style.color = '#ffd66b';
            panel.style.fontSize = '12px';
            panel.style.display = 'none';
            container.appendChild(panel);
          }
          return panel;
        };
        type DetailFeature = { properties?: { name_hj?: string; name_cn?: string; hip?: number|string; mag?: number; gua?: string; fenye?: string; note?: string; tags?: string[] } };
        const showDetail = (feat: DetailFeature & { properties?: { source_refs?: string[] } }) => {
          const panel = ensureDetail();
          const p = feat?.properties || {};
          panel!.innerHTML = `
            <div style="display:flex;justify-content:space-between;align-items:center;margin-bottom:6px;">
              <div style="font-weight:bold;color:#ffeaa7;">${p.name_hj || p.name_cn || '皇极星'}</div>
              <button id="hj-detail-close" style="background:transparent;color:#ffeaa7;border:1px solid #ffeaa7;border-radius:4px;font-size:11px;padding:2px 6px;">关闭</button>
            </div>
            <div>HIP：${p.hip ?? '-'}</div>
            <div>星等：${p.mag ?? '-'}</div>
            <div>卦象：${p.gua ?? '-'}</div>
            <div>分野：${p.fenye ?? '-'}</div>
            <div style="margin-top:6px;color:#f6e58d;">${p.note ?? ''}</div>
            <div style="margin-top:4px;color:#f6e58d;">${Array.isArray(p.tags) ? p.tags.join('、') : ''}</div>
            <div style="margin-top:4px;color:#999;">来源：${Array.isArray(p.source_refs) ? p.source_refs.join('；') : '—'}</div>
          `;
          panel!.style.display = 'block';
          const btn = panel!.querySelector('#hj-detail-close') as HTMLButtonElement | null;
          if (btn) btn.onclick = () => { panel!.style.display = 'none'; };
        };
        const onEnter = (e: MouseEvent) => {
          const rect = container.getBoundingClientRect();
          const target = e.currentTarget as SVGElement & { __data__?: DetailFeature };
          const feat = (target as { __data__?: DetailFeature }).__data__;
          const name = feat?.properties?.name_hj || feat?.properties?.name_cn || '';
          tooltip.textContent = name || '皇极星';
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
        const onClick = (e: MouseEvent) => {
          const target = e.currentTarget as SVGElement & { __data__?: DetailFeature };
          const feat = (target as { __data__?: DetailFeature }).__data__;
          if (feat) showDetail(feat);
        };
        points.forEach(p => {
          p.addEventListener('mouseenter', onEnter);
          p.addEventListener('mousemove', onMove);
          p.addEventListener('mouseleave', onLeave);
          p.addEventListener('click', onClick);
          (p as SVGElement).style.stroke = 'rgba(255,214,107,0.5)';
          (p as SVGElement).style.strokeWidth = '1.2px';
        });
      }, 1000);
      if (!initializedRef.current) {
        const restore = () => { if (cleanup) cleanup(); };
        window.__celestialTooltipCleanup = restore;
      }
      applyOnlyHjVisibility(onlyHj);
    }

    return () => {
      if (retryHandle) { try { clearTimeout(retryHandle); } catch { void 0; } }
      // 不主动 abort 设置拉取，避免预览环境产生 ERR_ABORTED
      const fn = window.__celestialTooltipCleanup;
      if (typeof fn === 'function') fn();
      try { window.alert = originalAlert; } catch { void 0; }
      if (playTimerRef.current) { try { clearInterval(playTimerRef.current); } catch { /* ignore */ } playTimerRef.current = null; }
    };
  }, [date, lat, lon, actualLat, actualLon, timezone, containerId, showXiu, renameZh, showConst, culture, onlyHj, playing, gpsLocation]);

  

  return (
    <div style={{ width: '100%', height: '100%', position: 'relative', background: '#000' }}>
      <div id={containerId} style={{ minHeight: 300 }}></div>
      <div style={{ position: 'absolute', right: 8, top: 8, display: 'flex', alignItems: 'center', gap: 8, background: 'rgba(0,0,0,0.4)', padding: '6px 8px', borderRadius: 6 }}>
        <label style={{ color: '#f6e58d', fontSize: 11, display: 'flex', alignItems: 'center', height: 26 }}>
          <input type="checkbox" checked={showConst} onChange={e => { const v = e.target.checked; setShowConst(v); const API_BASE = (import.meta as unknown as { env?: Record<string, string> }).env?.VITE_BACKEND_URL || ''; const USE_REMOTE_SETTINGS = false; if (API_BASE && USE_REMOTE_SETTINGS) { try { void fetch(`${API_BASE}/api/settings/sky`, { method: 'POST', headers: { 'Content-Type': 'application/json' }, body: JSON.stringify({ show_const: v, show_xiu: showXiu, zh_planet_names: renameZh, culture }), keepalive: true }); } catch { void 0; } } }} /> 星官
        </label>
        <label style={{ color: '#f6e58d', fontSize: 11, display: 'flex', alignItems: 'center', height: 26 }}>
          <input type="checkbox" checked={showXiu} onChange={e => { const v = e.target.checked; setShowXiu(v); const API_BASE = (import.meta as unknown as { env?: Record<string, string> }).env?.VITE_BACKEND_URL || ''; const USE_REMOTE_SETTINGS = false; if (API_BASE && USE_REMOTE_SETTINGS) { try { void fetch(`${API_BASE}/api/settings/sky`, { method: 'POST', headers: { 'Content-Type': 'application/json' }, body: JSON.stringify({ show_const: showConst, show_xiu: v, zh_planet_names: renameZh, culture }), keepalive: true }); } catch { void 0; } } }} /> 二十八宿
        </label>
        <label style={{ color: '#f6e58d', fontSize: 11, display: 'flex', alignItems: 'center', height: 26 }}>
          <input type="checkbox" checked={renameZh} onChange={e => { const v = e.target.checked; setRenameZh(v); const API_BASE = (import.meta as unknown as { env?: Record<string, string> }).env?.VITE_BACKEND_URL || ''; const USE_REMOTE_SETTINGS = false; if (API_BASE && USE_REMOTE_SETTINGS) { try { void fetch(`${API_BASE}/api/settings/sky`, { method: 'POST', headers: { 'Content-Type': 'application/json' }, body: JSON.stringify({ show_const: showConst, show_xiu: showXiu, zh_planet_names: v, culture }), keepalive: true }); } catch { void 0; } } }} /> 行星中文
        </label>
        <label style={{ color: '#f6e58d', fontSize: 11, display: 'flex', alignItems: 'center', gap: 4 }}>
          文化
          <select value={culture} onChange={e => { const v = e.target.value as ('cn'|'hj'|'iau'); setCulture(v); const API_BASE = (import.meta as unknown as { env?: Record<string, string> }).env?.VITE_BACKEND_URL || ''; const USE_REMOTE_SETTINGS = false; if (API_BASE && USE_REMOTE_SETTINGS) { try { void fetch(`${API_BASE}/api/settings/sky`, { method: 'POST', headers: { 'Content-Type': 'application/json' }, body: JSON.stringify({ show_const: showConst, show_xiu: showXiu, zh_planet_names: renameZh, culture: v }), keepalive: true }); } catch { void 0; } } }} style={{ background: 'transparent', color: '#f6e58d', border: '1px solid #f6e58d', borderRadius: 4, fontSize: 11, height: 26 }}>
            <option value="cn">中国</option>
            <option value="hj">皇极</option>
            <option value="iau">IAU</option>
          </select>
        </label>
        <label style={{ color: '#f6e58d', fontSize: 11, display: 'flex', alignItems: 'center', height: 26 }}>
          <input type="checkbox" checked={onlyHj} onChange={e => { const v = e.target.checked; setOnlyHj(v); const container = document.getElementById(containerId); if (!container) return; const svg = container.querySelector('svg'); if (!svg) return; const whites = Array.from((svg.querySelectorAll('circle') || []) as NodeListOf<SVGCircleElement>).filter(c => getComputedStyle(c).fill === 'rgb(255, 255, 255)'); const cnPts = Array.from((svg.querySelectorAll('circle') || []) as NodeListOf<SVGCircleElement>).filter(c => getComputedStyle(c).fill === 'rgb(246, 229, 141)'); const labels = Array.from((svg.querySelectorAll('text') || []) as NodeListOf<SVGTextElement>).filter(t => getComputedStyle(t).fill === 'rgb(246, 229, 141)'); const all = [...whites, ...cnPts, ...labels]; all.forEach(el => { el.style.opacity = v ? '0' : ''; }); }} /> 只看皇极星
        </label>
        <input value={searchTerm} onChange={e => {
          const v = e.target.value;
          setSearchTerm(v);
          const containerEl = document.getElementById(containerId);
          const svg = containerEl?.querySelector('svg');
          if (!svg) return;
          const points = Array.from((svg.querySelectorAll('.hj-star') || []) as NodeListOf<SVGElement>);
          points.forEach(el => {
            const feat = (el as { __data__?: { properties?: { name_hj?: string; name_cn?: string; hip?: number|string } } }).__data__;
            const name = feat?.properties?.name_hj || feat?.properties?.name_cn || '';
            const hip = String(feat?.properties?.hip || '');
            const hit = v && (name.includes(v) || hip.includes(v));
            el.style.opacity = hit || !v ? '1' : '0.15';
            el.style.strokeWidth = hit ? '2px' : '1.2px';
          });
        }} placeholder={"搜索皇极星/HIP"} style={{ background: 'transparent', color: '#f6e58d', border: '1px solid #f6e58d', borderRadius: 4, fontSize: 11, padding: '2px 6px', height: 26 }} />
        <select 
          value={playSpeed} 
          onChange={e => setPlaySpeed(Number(e.target.value))}
          style={{ background: 'transparent', color: '#f6e58d', border: '1px solid #f6e58d', borderRadius: 4, fontSize: 10, height: 26, width: 65 }}
          title="时间流速"
        >
          <option value="1">1x 实时</option>
          <option value="60">60x</option>
          <option value="360">6分/秒</option>
          <option value="3600">1时/秒</option>
          <option value="86400">1天/秒</option>
        </select>
        <button onClick={() => {
          const next = !playing; 
          setPlaying(next);
          if (next) {
            if (playTimerRef.current) { try { clearInterval(playTimerRef.current); } catch { /* ignore */ } }
            const startDate = date instanceof Date && !isNaN(date.getTime()) ? date : new Date();
            playStartRef.current = { realTime: Date.now(), simTime: startDate };
            
            // 使用较短的间隔实现平滑旋转
            playTimerRef.current = window.setInterval(() => {
              if (!playStartRef.current) return;
              const elapsed = Date.now() - playStartRef.current.realTime;
              const simElapsed = elapsed * playSpeed; // 模拟经过的毫秒数
              const simDate = new Date(playStartRef.current.simTime.getTime() + simElapsed);
              const tzMinutes = Number.isFinite(timezone) ? Math.round(timezone * 60) : 0;
              try {
                if (window.Celestial && typeof window.Celestial.skyview === 'function') {
                  window.Celestial.skyview({ date: simDate, location: [actualLat, actualLon], timezone: tzMinutes });
                }
              } catch { void 0; }
            }, 100); // 每100ms更新一次，实现平滑效果
          } else {
            if (playTimerRef.current) { try { clearInterval(playTimerRef.current); } catch { /* ignore */ } playTimerRef.current = null; }
            playStartRef.current = null;
          }
        }} style={{ background: playing ? '#f6e58d22' : 'transparent', color: '#f6e58d', border: '1px solid #f6e58d', borderRadius: 4, fontSize: 11, height: 26, minWidth: 45 }}>
          {playing ? '⏸' : '▶'}
        </button>
        <button 
          onClick={requestGpsLocation}
          disabled={gpsLoading}
          style={{ background: gpsLocation ? '#4ade8022' : 'transparent', color: gpsLocation ? '#4ade80' : '#f6e58d', border: `1px solid ${gpsLocation ? '#4ade80' : '#f6e58d'}`, borderRadius: 4, fontSize: 11, height: 26, minWidth: 35 }}
          title={gpsLocation ? `GPS: ${gpsLocation.lat.toFixed(2)}°, ${gpsLocation.lon.toFixed(2)}°` : '获取GPS定位'}
        >
          {gpsLoading ? '...' : '📍'}
        </button>
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
