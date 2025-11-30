import React from 'react';
import { StarMap } from '../StarMap';
import type { SkyResponse } from '../types';

interface SkyCardProps {
  data: SkyResponse;
  date: Date;
  lat: number;
  lon: number;
  timezone?: number;
  containerId?: string;
}

const SkyCard: React.FC<SkyCardProps> = ({ data, date, lat, lon, timezone, containerId }) => {
  const sun = data.bodies.find(b => b.name === 'Sun');
  const moon = data.bodies.find(b => b.name === 'Moon');
  const planets = data.bodies.filter(b => !['Sun', 'Moon', 'Earth'].includes(b.name)).slice(0, 3);

  return (
    <div className="glass-panel rounded-3xl overflow-hidden flex flex-col h-full">
      <div className="p-6 border-b border-white/10 flex justify-between items-center bg-white/5">
        <h3 className="text-gold font-serif tracking-widest text-lg">星空 (Sky)</h3>
        <div className="text-xs text-gray-400 font-mono bg-black/20 px-2 py-1 rounded">
          {lat.toFixed(2)}°N, {lon.toFixed(2)}°E
        </div>
      </div>
      
      {/* Map Container - Flexible height */}
      <div className="relative flex-grow min-h-[300px] bg-black">
         <StarMap 
            date={date}
            lat={lat}
            lon={lon}
            _planets={data.bodies}
            timezone={timezone}
            containerId={containerId}
         />
         
         {/* Overlay Note */}
         <div className="absolute bottom-2 left-2 text-[10px] text-gray-600 bg-black/50 px-2 py-1 rounded pointer-events-none">
            Static Projection (Equatorial)
         </div>
      </div>

      {/* Data List */}
      <div className="bg-black/40 backdrop-blur-sm p-4 grid grid-cols-2 gap-4 text-xs font-mono border-t border-white/10">
         {sun && (
           <div className="flex justify-between items-center text-yellow-200/80">
              <span className="flex items-center gap-2"><span className="w-2 h-2 rounded-full bg-yellow-500"></span> Sun</span>
              <span>Alt {sun.alt_deg.toFixed(1)}° / Az {sun.az_deg.toFixed(1)}°</span>
           </div>
         )}
         {moon && (
           <div className="flex justify-between items-center text-gray-300">
              <span className="flex items-center gap-2"><span className="w-2 h-2 rounded-full bg-gray-400"></span> Moon</span>
              <span>Alt {moon.alt_deg.toFixed(1)}° / Az {moon.az_deg.toFixed(1)}°</span>
           </div>
         )}
         {planets.map(p => (
           <div key={p.name} className="flex justify-between items-center text-cyan-300/70 col-span-2 sm:col-span-1">
              <span className="flex items-center gap-2"><span className="w-1.5 h-1.5 rounded-full bg-cyan-500"></span> {p.name}</span>
              <span>Alt {p.alt_deg.toFixed(1)}°</span>
           </div>
         ))}
      </div>
    </div>
  );
};

export default SkyCard;
