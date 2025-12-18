/**
 * 时区转换工具函数
 * 
 * 符号约定：
 * tzOffsetMinutes: 时区偏移（分钟），东为正 UTC+8=+480, 西为负 UTC-5=-300
 * 注意：与 JS Date.getTimezoneOffset() 符号相反！
 */

/**
 * 将本地时间字符串根据指定时区偏移转换为 UTC ISO 字符串
 * 
 * 关键设计：显式使用用户选择的时区偏移，不依赖浏览器时区
 * 
 * @param localDateTime - 本地时间字符串，格式: "2025-12-05T02:48" (无时区信息)
 * @param tzOffsetMinutes - 时区偏移（分钟），东为正 UTC+8=+480, 西为负 UTC-5=-300
 *                          注意：与 JS Date.getTimezoneOffset() 符号相反！
 * @returns UTC ISO8601 字符串
 * 
 * 使用说明：
 * - 用户输入的时间被理解为"选择的时区的时间"
 * - 无论浏览器在哪个时区，选择"北京时间"并输入"02:48"，结果都是一致的
 * - 转换公式：UTC = 本地时间 - 时区偏移
 *   例如：北京时间 21:48 (UTC+8) → UTC 13:48
 */
export function convertLocalToUTC(localDateTime: string, tzOffsetMinutes: number): string {
  if (!localDateTime) {
    return new Date().toISOString();
  }
  
  try {
    // localDateTime 格式: "2025-12-05T02:48" (没有时区信息)
    const [datePart, timePart] = localDateTime.split('T');
    const [year, month, day] = datePart.split('-').map(Number);
    const [hour, minute] = (timePart || '00:00').split(':').map(Number);
    
    // 核心逻辑：使用 Date.UTC() 构造 UTC 时间
    // 用户输入的时间被理解为"目标时区的时间"
    // 转换公式：UTC = 本地时间 - 时区偏移
    // tzOffsetMinutes: 东为正 UTC+8=+480, 西为负 UTC-5=-300
    const totalMinutes = hour * 60 + minute - tzOffsetMinutes;
    const utcHour = Math.floor(totalMinutes / 60);
    const utcMinute = ((totalMinutes % 60) + 60) % 60; // 处理负数取模
    
    // 使用 Date.UTC() 构造 UTC 时间，自动处理日期跨越
    const utcDate = new Date(Date.UTC(
      year,
      month - 1,
      day,
      utcHour,
      utcMinute,
      0
    ));
    
    return utcDate.toISOString();
  } catch (error) {
    console.error('时区转换失败:', error);
    // 降级：返回当前时间
    return new Date().toISOString();
  }
}

/**
 * 根据时区名称获取时区偏移（分钟）
 * tzOffsetMinutes: 东为正 UTC+8=+480, 西为负 UTC-5=-300
 * 注意：与 JS Date.getTimezoneOffset() 符号相反！
 */
export function getTimezoneOffsetMinutes(timezoneName: string): number {
  const timezoneOffsets: Record<string, number> = {
    'Asia/Shanghai': 480,      // UTC+8 = +480 分钟
    'Asia/Tokyo': 540,         // UTC+9 = +540 分钟
    'Asia/Hong_Kong': 480,     // UTC+8 = +480 分钟
    'Asia/Taipei': 480,        // UTC+8 = +480 分钟
    'Asia/Singapore': 480,     // UTC+8 = +480 分钟
    'America/New_York': -300,  // UTC-5 = -300 分钟 (EST，不考虑DST)
    'America/Los_Angeles': -480, // UTC-8 = -480 分钟 (PST)
    'Europe/London': 0,        // UTC+0 = 0 分钟 (GMT)
    'Europe/Paris': 60,        // UTC+1 = +60 分钟 (CET)
  };
  return timezoneOffsets[timezoneName] ?? 480; // 默认北京时间
}
