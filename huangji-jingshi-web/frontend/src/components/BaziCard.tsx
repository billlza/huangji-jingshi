/**
 * BaziCard - 人生命盘卡片
 * 整合 BaziForm 和 BaziChartView
 */

import { useState } from 'react';
import BaziForm, { type BaziParams } from './BaziForm';
import BaziChartView, { type BaziResult } from './BaziChartView';
import { Sparkles } from 'lucide-react';

interface BaziCardProps {
  // 从父组件传入的观测时间参数
  observeParams: {
    datetime: string;
    lat: number;
    lon: number;
  };
}

export default function BaziCard({ observeParams }: BaziCardProps) {
  const API_BASE = import.meta.env.VITE_BACKEND_URL || '';
  const [result, setResult] = useState<BaziResult | null>(null);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const handleSubmit = async (params: BaziParams) => {
    setLoading(true);
    setError(null);
    
    try {
      const q = new URLSearchParams({
        datetime: params.datetime,
        timezone: params.timezone,
        lat: params.lat.toString(),
        lon: params.lon.toString(),
        gender: params.gender
      });
      
      const res = await fetch(`${API_BASE}/api/bazi?${q}`);
      
      if (!res.ok) {
        const errText = await res.text();
        throw new Error(errText || '排盘失败');
      }
      
      const data = await res.json();
      setResult(data);
    } catch (err) {
      console.error('Bazi API Error:', err);
      setError(err instanceof Error ? err.message : '排盘失败，请稍后重试');
      
      // 使用模拟数据作为fallback（开发/演示用）
      setResult(generateMockBazi(params));
    } finally {
      setLoading(false);
    }
  };

  return (
    <div className="glass-panel rounded-3xl overflow-hidden">
      {/* 标题栏 */}
      <div className="px-6 py-4 border-b border-white/5 flex items-center gap-3">
        <div className="w-8 h-8 rounded-full bg-gradient-to-br from-amber-500/30 to-red-500/30 flex items-center justify-center">
          <Sparkles className="w-4 h-4 text-gold" />
        </div>
        <div>
          <h3 className="text-sm font-bold text-white">人生命盘</h3>
          <p className="text-[10px] text-gray-500 uppercase tracking-widest">Birth Chart · 八字排盘</p>
        </div>
      </div>

      {/* 内容区域 */}
      <div className="p-6">
        <div className="grid grid-cols-1 lg:grid-cols-2 gap-6">
          {/* 左侧：输入表单 */}
          <div>
            <BaziForm
              observeParams={observeParams}
              onSubmit={handleSubmit}
              isLoading={loading}
            />
            
            {error && (
              <div className="mt-4 p-3 rounded-xl bg-red-500/10 border border-red-500/20 text-red-300 text-xs">
                {error}
              </div>
            )}
          </div>
          
          {/* 右侧：四柱结果 */}
          <div>
            <BaziChartView
              data={result}
              isLoading={loading}
            />
          </div>
        </div>
      </div>
    </div>
  );
}

/**
 * 生成模拟八字数据（用于演示或API不可用时）
 */
function generateMockBazi(params: BaziParams): BaziResult {
  const date = new Date(params.datetime);
  const year = date.getFullYear();
  
  // 简化的天干地支计算（仅用于演示）
  const TIANGAN = ['甲', '乙', '丙', '丁', '戊', '己', '庚', '辛', '壬', '癸'];
  const DIZHI = ['子', '丑', '寅', '卯', '辰', '巳', '午', '未', '申', '酉', '戌', '亥'];
  const WUXING_GAN: Record<string, string> = {
    '甲': '阳木', '乙': '阴木', '丙': '阳火', '丁': '阴火', '戊': '阳土',
    '己': '阴土', '庚': '阳金', '辛': '阴金', '壬': '阳水', '癸': '阴水'
  };
  const WUXING_ZHI: Record<string, string> = {
    '子': '阳水', '丑': '阴土', '寅': '阳木', '卯': '阴木', '辰': '阳土', '巳': '阴火',
    '午': '阳火', '未': '阴土', '申': '阳金', '酉': '阴金', '戌': '阳土', '亥': '阴水'
  };
  const ANIMALS = ['鼠', '牛', '虎', '兔', '龙', '蛇', '马', '羊', '猴', '鸡', '狗', '猪'];
  const NAYIN = [
    '海中金', '炉中火', '大林木', '路旁土', '剑锋金', '山头火',
    '涧下水', '城头土', '白蜡金', '杨柳木', '泉中水', '屋上土',
    '霹雳火', '松柏木', '长流水', '砂石金', '山下火', '平地木',
    '壁上土', '金箔金', '覆灯火', '天河水', '大驿土', '钗钏金',
    '桑柘木', '大溪水', '沙中土', '天上火', '石榴木', '大海水'
  ];

  // 年柱计算
  const yearGanIdx = (year - 4) % 10;
  const yearZhiIdx = (year - 4) % 12;
  
  // 月柱（简化计算）
  const month = date.getMonth() + 1;
  const monthGanIdx = (yearGanIdx * 2 + month) % 10;
  const monthZhiIdx = (month + 1) % 12;
  
  // 日柱（简化计算）
  const dayOfYear = Math.floor((date.getTime() - new Date(year, 0, 0).getTime()) / 86400000);
  const dayGanIdx = (year + dayOfYear) % 10;
  const dayZhiIdx = (year + dayOfYear + 2) % 12;
  
  // 时柱
  const hour = date.getHours();
  const hourZhiIdx = Math.floor((hour + 1) / 2) % 12;
  const hourGanIdx = (dayGanIdx * 2 + hourZhiIdx) % 10;

  const createPillar = (ganIdx: number, zhiIdx: number, nayinOffset: number) => {
    const gan = TIANGAN[ganIdx];
    const zhi = DIZHI[zhiIdx];
    return {
      gan,
      zhi,
      gan_wuxing: WUXING_GAN[gan],
      zhi_wuxing: WUXING_ZHI[zhi],
      zhi_animal: ANIMALS[zhiIdx],
      nayin: NAYIN[(ganIdx * 6 + zhiIdx + nayinOffset) % 30]
    };
  };

  const yearPillar = createPillar(yearGanIdx, yearZhiIdx, 0);
  const monthPillar = createPillar(monthGanIdx, monthZhiIdx, 1);
  const dayPillar = createPillar(dayGanIdx, dayZhiIdx, 2);
  const hourPillar = createPillar(hourGanIdx, hourZhiIdx, 3);

  // 统计五行
  const wuxingCounts: Record<string, number> = { '木': 0, '火': 0, '土': 0, '金': 0, '水': 0 };
  [yearPillar, monthPillar, dayPillar, hourPillar].forEach(p => {
    const ganWx = p.gan_wuxing.replace(/[阴阳]/g, '');
    const zhiWx = p.zhi_wuxing.replace(/[阴阳]/g, '');
    wuxingCounts[ganWx] = (wuxingCounts[ganWx] || 0) + 1;
    wuxingCounts[zhiWx] = (wuxingCounts[zhiWx] || 0) + 1;
  });

  const dayMasterWx = dayPillar.gan_wuxing.replace(/[阴阳]/g, '');
  const dayMasterCount = wuxingCounts[dayMasterWx];
  const totalCount = Object.values(wuxingCounts).reduce((a, b) => a + b, 0);
  
  return {
    year_pillar: yearPillar,
    month_pillar: monthPillar,
    day_pillar: dayPillar,
    hour_pillar: hourPillar,
    wuxing_analysis: {
      day_master: dayPillar.gan_wuxing,
      day_master_strength: dayMasterCount >= 3 ? 'strong' : dayMasterCount <= 1 ? 'weak' : 'balanced',
      wuxing_counts: wuxingCounts,
      missing_wuxing: Object.entries(wuxingCounts)
        .filter(([_, count]) => count === 0)
        .map(([wx]) => wx)
    },
    gender: params.gender,
    zodiac: ANIMALS[yearZhiIdx]
  };
}

