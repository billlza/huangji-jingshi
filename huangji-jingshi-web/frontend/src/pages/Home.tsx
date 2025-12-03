
import { Link } from 'react-router-dom';
import StarField from '../components/StarField';
import { Compass, BookOpen, ArrowRight } from 'lucide-react';

export default function Home() {
  return (
    <div className="min-h-screen bg-[#050508] text-white font-sans relative overflow-hidden flex flex-col">
      {/* Nebula Background */}
      <div className="nebula-container">
        <div className="nebula-layer nebula-1"></div>
        <div className="nebula-layer nebula-2"></div>
        <div className="nebula-layer nebula-3"></div>
      </div>

      {/* StarField Overlay */}
      <div className="fixed inset-0 z-0 opacity-70 mix-blend-screen">
        <StarField />
      </div>

      {/* Hero Section */}
      <main className="relative z-10 flex-1 flex flex-col items-center justify-center px-4 text-center">
        
        <div className="mb-12 relative">
           {/* Subtle Glow behind Title */}
           <div className="absolute left-1/2 top-1/2 -translate-x-1/2 -translate-y-1/2 w-[600px] h-[200px] bg-gold blur-[120px] opacity-10 rounded-full pointer-events-none"></div>
           
           <h1 className="text-6xl md:text-8xl font-serif font-bold text-transparent bg-clip-text bg-gradient-to-b from-[#FCEda3] via-[#D4AF37] to-[#8a7020] tracking-[0.15em] mb-6 drop-shadow-2xl">
             皇极经世
           </h1>
           <div className="h-px w-32 mx-auto bg-gradient-to-r from-transparent via-gold/50 to-transparent mb-6"></div>
           <p className="text-lg md:text-xl text-gray-400 font-light tracking-[0.6em] uppercase">
             Cosmic Chronology
           </p>
        </div>

        <p className="max-w-2xl mx-auto text-gray-300 leading-relaxed mb-16 text-lg font-light tracking-wide">
          探究宇宙宏观演化，推演历史兴衰治乱。<br/>
          基于北宋邵雍《皇极经世书》构建的数字化推演平台。
        </p>

        <div className="flex flex-col sm:flex-row gap-8 w-full max-w-xl mx-auto">
          <Link 
            to="/tools" 
            className="group flex-1 flex items-center justify-center gap-3 btn-glass-primary px-8 py-4 rounded-full font-bold text-lg"
          >
            <Compass className="w-5 h-5 group-hover:rotate-45 transition-transform duration-500" />
            <span>开始推演</span>
            <ArrowRight className="w-4 h-4 opacity-0 -translate-x-2 group-hover:opacity-100 group-hover:translate-x-0 transition-all duration-300" />
          </Link>
          
          <Link 
            to="/story" 
            className="group flex-1 flex items-center justify-center gap-3 btn-glass-secondary px-8 py-4 rounded-full font-medium text-lg"
          >
            <BookOpen className="w-5 h-5 text-gold/80 group-hover:text-gold transition-colors" />
            <span>了解背景</span>
          </Link>

          <Link 
            to="/about" 
            className="group flex-1 flex items-center justify-center gap-3 btn-glass-secondary px-8 py-4 rounded-full font-medium text-lg"
          >
            <BookOpen className="w-5 h-5 text-gold/80 group-hover:text-gold transition-colors" />
            <span>关于/数据来源</span>
          </Link>
        </div>

        {/* Features Grid - Liquid Glass */}
        <div className="grid grid-cols-1 md:grid-cols-3 gap-8 mt-32 max-w-6xl mx-auto text-left px-4">
           <div className="glass-panel p-8 rounded-3xl">
              <div className="text-gold text-2xl font-serif mb-4 border-l-2 border-gold/50 pl-3">元会运世</div>
              <p className="text-gray-400 text-sm leading-relaxed">精确计算当前的宇宙时间坐标，解析大尺度的时空定位。</p>
           </div>
           <div className="glass-panel p-8 rounded-3xl">
              <div className="text-gold text-2xl font-serif mb-4 border-l-2 border-gold/50 pl-3">值年卦象</div>
              <p className="text-gray-400 text-sm leading-relaxed">推导每一年的流年卦象，洞察年度气运与人事吉凶。</p>
           </div>
           <div className="glass-panel p-8 rounded-3xl">
              <div className="text-gold text-2xl font-serif mb-4 border-l-2 border-gold/50 pl-3">星图参佐</div>
              <p className="text-gray-400 text-sm leading-relaxed">结合实时天象星图，实现天人合一的立体化推演。</p>
           </div>
        </div>

      </main>

      <footer className="relative z-10 py-8 text-center text-gray-600 text-xs">
        <p>© {new Date().getFullYear()} Huangji Jingshi Platform. Open Source Project.</p>
      </footer>
    </div>
  );
}
