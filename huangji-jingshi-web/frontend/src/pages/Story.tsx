
import StarField from '../components/StarField';

export default function Story() {
  return (
    <div className="min-h-screen bg-gray-950 text-gray-200 font-sans relative overflow-hidden">
      <div className="fixed inset-0 z-0 opacity-20 pointer-events-none">
        <StarField />
      </div>

      <div className="relative z-10 max-w-4xl mx-auto p-6 md:p-12">
        <header className="mb-16 text-center">
           <div className="inline-block w-4 h-4 bg-gold rotate-45 mb-6 shadow-[0_0_10px_#D4AF37]"></div>
           <h1 className="text-4xl md:text-5xl font-serif text-gold mb-4 tracking-widest">皇极经世</h1>
           <p className="text-gray-500 uppercase tracking-[0.3em] text-sm">The Book of Supreme World Ordering Principles</p>
        </header>

        <article className="prose prose-invert prose-lg mx-auto space-y-12">
           <section>
              <h2 className="text-2xl font-serif text-gray-100 border-l-4 border-gold pl-4 mb-6">缘起与背景</h2>
              <p className="text-gray-300 leading-relaxed">
                《皇极经世书》是北宋著名理学家邵雍（邵康节）毕生研究周易而自创的经天纬地之预测学著作。
                邵雍隐居洛阳三十年，究天人之际，通古今之变，创立了“元、会、运、世”一套有规律的预测方法，
                试图揭示宇宙起源、自然演化和社会历史变迁的宏大规律。
              </p>
           </section>

           <section>
              <h2 className="text-2xl font-serif text-gray-100 border-l-4 border-gold pl-4 mb-6">元会运世：时间的代码</h2>
              <div className="bg-white/5 p-6 rounded-xl border border-white/10 my-8">
                <div className="grid grid-cols-2 md:grid-cols-4 gap-4 text-center">
                   <div className="p-4">
                      <div className="text-gold text-3xl font-serif mb-2">元</div>
                      <div className="text-xs text-gray-500 uppercase">1 Yuan</div>
                      <div className="text-sm mt-2 text-gray-300">129,600 年</div>
                   </div>
                   <div className="p-4 border-l border-white/10">
                      <div className="text-gold text-3xl font-serif mb-2">会</div>
                      <div className="text-xs text-gray-500 uppercase">1 Hui</div>
                      <div className="text-sm mt-2 text-gray-300">10,800 年</div>
                   </div>
                   <div className="p-4 border-l border-white/10">
                      <div className="text-gold text-3xl font-serif mb-2">运</div>
                      <div className="text-xs text-gray-500 uppercase">1 Yun</div>
                      <div className="text-sm mt-2 text-gray-300">360 年</div>
                   </div>
                   <div className="p-4 border-l border-white/10">
                      <div className="text-gold text-3xl font-serif mb-2">世</div>
                      <div className="text-xs text-gray-500 uppercase">1 Shi</div>
                      <div className="text-sm mt-2 text-gray-300">30 年</div>
                   </div>
                </div>
              </div>
              <p className="text-gray-300 leading-relaxed">
                邵雍认为，宇宙的演化如同一年四季的更替，而“元”就是宇宙的大年。
                一元分为十二会（子、丑、寅、卯...），每会对应不同的演化阶段。
                我们目前正处于“午”会，是人类文明极盛的时期。
              </p>
           </section>

           <section>
              <h2 className="text-2xl font-serif text-gray-100 border-l-4 border-gold pl-4 mb-6">以卦配时</h2>
              <p className="text-gray-300 leading-relaxed">
                《皇极经世》的核心在于“以卦配时”。邵雍将六十四卦（除去乾、坤、坎、离四正卦，余六十卦）
                分配给不同的时间单位。通过计算当前的元、会、运、世对应的卦象，
                可以推演此时此刻的天道气数与人事兴衰。
              </p>
              <p className="text-gray-300 leading-relaxed mt-4">
                每一个年份都有其对应的“值年卦”，每一个“运”也有其对应的“值运卦”。
                这种独特的时间坐标系，为我们提供了一种全新的视角来审视历史与未来。
              </p>
           </section>
        </article>

        <footer className="mt-24 pt-12 border-t border-white/10 text-center">
           <p className="text-gray-500 text-sm italic">
             “安乐窝中一部书，号云皇极意如何。中间三千年，迄今之陈迹。” —— 邵雍
           </p>
        </footer>
      </div>
    </div>
  );
}
