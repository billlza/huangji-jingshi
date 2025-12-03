import React from 'react';

interface HexagramProps {
  name: string;
  code?: number[]; // Array of 6 numbers (0 or 1), from TOP to BOTTOM
}

const Hexagram: React.FC<HexagramProps> = ({ name, code }) => {
  const displayCode = code || [1, 1, 1, 1, 1, 1]; 

  return (
    <div className="flex flex-col items-center justify-center p-4 relative group cursor-pointer">
      {/* Glow effect behind */}
      <div className="absolute inset-0 bg-gold/5 blur-3xl rounded-full opacity-0 group-hover:opacity-100 transition-opacity duration-1000"></div>

      <div className="flex flex-col gap-2 w-24 mb-4 z-10">
        {displayCode.map((val, idx) => (
          <div 
            key={idx} 
            className={`h-2 rounded-sm shadow-[0_0_8px_rgba(212,175,55,0.3)] transition-all duration-500 group-hover:shadow-[0_0_15px_rgba(212,175,55,0.6)] ${val === 1 ? 'bg-gold w-full' : 'flex justify-between w-full'}`}
          >
             {val === 0 && (
                <>
                    <div className="w-[45%] h-full bg-gold rounded-sm shadow-[0_0_8px_rgba(212,175,55,0.3)]"></div>
                    <div className="w-[45%] h-full bg-gold rounded-sm shadow-[0_0_8px_rgba(212,175,55,0.3)]"></div>
                </>
             )}
          </div>
        ))}
      </div>
      
      <div className="relative z-10 text-center">
         <h2 className="text-5xl font-serif text-gold tracking-widest drop-shadow-[0_2px_4px_rgba(0,0,0,0.8)] group-hover:text-white transition-colors duration-500">
            {name}
         </h2>
         <div className="h-[1px] w-12 bg-gold/50 mx-auto my-2"></div>
         <span className="text-[10px] uppercase tracking-[0.3em] text-gold/60">Annual Hexagram</span>
      </div>
    </div>
  );
};

export default Hexagram;
