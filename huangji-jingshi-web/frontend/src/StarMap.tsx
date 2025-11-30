
import { useEffect, useRef } from 'react';

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
  useEffect(() => {
    if (!window.Celestial) return;
    const config = {
      width: 0, // Full width
      projection: "airy", 
      transform: "equatorial", 
      center: null, // [0,0,0]
      adaptable: true,
      interactive: true,
      form: false, // Disable built-in form
      location: false, // Disable built-in geolocation to prevent TimeZoneDB calls
      controls: false,
      lang: "zh", // Chinese
      culture: "cn", // Switch to Traditional Chinese constellations (28 Mansions)
      container: containerId,
      datapath: "https://ofrohn.github.io/data/", // Use remote data for now to ensure loading
      stars: {
        show: true,
        limit: 6,
        colors: true,
        style: { fill: "#ffffff", opacity: 1 },
        names: false, // Show names
        propername: false,
        designation: false,
      },
      dsos: { show: false },
      constellations: {
        show: true,
        names: false, 
        desig: false, // IAU designation not relevant for Chinese
        lines: true, 
        linestyle: { stroke: "#cccccc", width: 1.5, opacity: 0.7 }, // Slightly thicker lines
        namesType: "name", // Use the name provided by the culture file
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
          sun: { scale: 1.5, fill: "#ffff00", stroke: "#cccc00" },
          moon: { scale: 1.5, fill: "#eeeeee", stroke: "#cccccc" },
          planet: { scale: 1.2, fill: "#00f3ff", stroke: "#0099aa" },
        },
        names: true
      },
      horizon: {
        show: true, 
        stroke: "#000000", 
        width: 3, 
        fill: "#000000", 
        opacity: 0.6 
      }, 
      daylight: { show: false }, // Show night sky for clarity
    };

    if (!initializedRef.current) {
      window.Celestial.display(config);
    }
    const tzMinutes = Number.isFinite(timezone) ? Math.round(timezone * 60) : 0;
    const safeDate = date instanceof Date && !isNaN(date.getTime()) ? date : new Date();
    window.Celestial.skyview({ date: safeDate, location: [lat, lon], timezone: tzMinutes });

    const container = document.getElementById(containerId);
    if (container) {
      const tooltip = document.createElement('div');
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
        const onLeave = () => { tooltip.style.display = 'none'; };
        eclPath.addEventListener('mouseenter', onEnter);
        eclPath.addEventListener('mousemove', onMove);
        eclPath.addEventListener('mouseleave', onLeave);
        cleanup = () => {
          eclPath.removeEventListener('mouseenter', onEnter);
          eclPath.removeEventListener('mousemove', onMove);
          eclPath.removeEventListener('mouseleave', onLeave);
          tooltip.remove();
        };
      }, 500);
      if (!initializedRef.current) {
        const restore = () => { if (cleanup) cleanup(); };
        window.__celestialTooltipCleanup = restore;
      }
    }

    initializedRef.current = true;
    return () => {
      if (!initializedRef.current) return;
      const fn = window.__celestialTooltipCleanup;
      if (typeof fn === 'function') fn();
    };
  }, [date, lat, lon, timezone, containerId]);

  

  return (
    <div style={{ width: '100%', height: '100%', position: 'relative', background: '#000' }}>
      <div id={containerId}></div>
      {/* Hidden stub form to satisfy d3-celestial internal updates (render once) */}
      {containerId === 'celestial-map' && (
        <div id="celestial-form" style={{ display: 'none' }}>
          <input type="text" id="datetime" defaultValue={new Date().toISOString()} />
        </div>
      )}
    </div>
  );
}
