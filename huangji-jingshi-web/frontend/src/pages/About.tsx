import { Link } from 'react-router-dom';
import StarField from '../components/StarField';

export default function About() {
  const year = new Date().getFullYear();
  return (
    <div className="min-h-screen bg-[#050508] text-white font-sans relative overflow-hidden flex flex-col">
      <div className="fixed inset-0 z-0 opacity-60 mix-blend-screen pointer-events-none">
        <StarField />
      </div>

      <header className="relative z-10 px-6 md:px-12 py-6 flex items-center justify-between">
        <Link to="/" className="text-xs text-gray-400 hover:text-gray-200 underline decoration-dotted">返回首页</Link>
        <div className="text-[10px] text-gray-500 font-mono">© {year} Huangji Jingshi</div>
      </header>

      <main className="relative z-10 flex-1 max-w-4xl mx-auto w-full px-6 md:px-12">
        <h1 className="text-3xl md:text-4xl font-serif text-gold tracking-widest mb-6">关于 · 数据来源</h1>

        <section className="glass-panel rounded-2xl p-6 border border-white/10 mb-8">
          <h2 className="text-xl font-serif text-gray-100 mb-4">算法依据</h2>
          <p className="text-gray-300 leading-relaxed text-sm">
            本站皇极经世算法依据：自编元会运世数表 + 公开古籍；以“以卦配时”为基本原则，结合时间坐标（元/会/运/世/旬）进行推演。
          </p>
        </section>

        <section className="glass-panel rounded-2xl p-6 border border-white/10 mb-8">
          <h2 className="text-xl font-serif text-gray-100 mb-4">古籍底本</h2>
          <p className="text-gray-300 leading-relaxed text-sm mb-3">
            《皇極經世書》（四库全书本），底本影印见浙江大学图书馆，电子文本参考：中国哲学书电子化计划（ctext.org）、维基文库（zh.wikisource.org）。
          </p>
          <div className="text-sm text-cyan-400 space-y-2">
            <div>
              原文入口（ctext）：
              <a href="https://ctext.org/wiki.pl?if=gb&res=561844" target="_blank" className="ml-1 hover:underline">《皇極經世書》总入口</a>
            </div>
            <div>
              维基文库：
              <a href="https://zh.wikisource.org/wiki/皇極經世" target="_blank" className="ml-1 hover:underline">《皇極經世》条目</a>
              <span className="mx-1">·</span>
              <a href="https://zh.wikisource.org/wiki/皇極經世書_(四庫全書本)" target="_blank" className="hover:underline">四库全书本专页</a>
            </div>
          </div>
        </section>

        <section className="glass-panel rounded-2xl p-6 border border-white/10 mb-8">
          <h2 className="text-xl font-serif text-gray-100 mb-4">数学与数表参考</h2>
          <p className="text-gray-300 leading-relaxed text-sm mb-3">
            海云青飞《皇極經世书：常用图表》《一元十等数表》等文章（tuenhai.com），仅用于算法校对。
          </p>
          <div className="text-sm text-cyan-400">
            <a href="https://tuenhai.com/tieban/huang-ji-fly-tables.html" target="_blank" className="hover:underline">《皇極經世书：常用图表》（海云青飞）</a>
          </div>
        </section>

        <section className="glass-panel rounded-2xl p-6 border border-white/10 mb-20">
          <h2 className="text-xl font-serif text-gray-100 mb-4">署名与使用说明</h2>
          <ul className="text-gray-300 leading-relaxed text-sm space-y-2">
            <li>文本整理参考：中国哲学书电子化计划（ctext.org）。</li>
            <li>部分古籍文本整理自维基文库《皇極經世書（四庫全書本）》，遵循 CC BY-SA 3.0 协议。</li>
            <li>本站不进行批量抓取或复制整表，仅作少量引用与链接，引导读者至原始来源。</li>
            <li>数表与方法仅作为校对标尺，不作为官方数据源发布。</li>
          </ul>
        </section>
      </main>

      <footer className="relative z-10 py-8 text-center text-gray-600 text-xs">
        <p>© {year} Huangji Jingshi Platform. Open Source Project.</p>
      </footer>
    </div>
  );
}

