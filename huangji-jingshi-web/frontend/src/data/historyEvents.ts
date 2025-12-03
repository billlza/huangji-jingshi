// History events data
export interface HistoryEvent {
  year: number;
  title: string;
  description: string;
}

export const HISTORY_EVENTS: HistoryEvent[] = [
  { year: 2008, title: "北京奥运会", description: "第29届夏季奥林匹克运动会在北京开幕" },
  { year: 2019, title: "新冠疫情爆发", description: "COVID-19 全球大流行开始" },
  { year: 2022, title: "北京冬奥会", description: "第24届冬季奥林匹克运动会在北京举办" },
  { year: 1997, title: "香港回归", description: "香港特别行政区成立" },
  { year: 1949, title: "新中国成立", description: "中华人民共和国成立" },
  { year: 1911, title: "辛亥革命", description: "推翻清朝统治，建立中华民国" },
  { year: 1840, title: "鸦片战争", description: "第一次鸦片战争爆发" },
  { year: 1644, title: "明朝灭亡", description: "清军入关，明朝灭亡" },
  // Add more events as needed
];
