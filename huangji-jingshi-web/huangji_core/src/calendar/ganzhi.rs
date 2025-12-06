//! 干支历法模块
//!
//! 提供天干地支计算，用于八字四柱排盘。
//!
//! 算法来源：
//! - 《子平真诠》
//! - 《三命通会》
//! - 传统命理学典籍

use crate::astro::solar::{solar_position, datetime_to_jd, hour_to_dizhi_index, true_solar_hour};
use crate::calendar::jieqi::{find_next_jie, find_prev_jie, SolarTerm};
use chrono::{DateTime, Datelike, Utc};

/// 天干
pub const TIANGAN: [&str; 10] = ["甲", "乙", "丙", "丁", "戊", "己", "庚", "辛", "壬", "癸"];

/// 地支
pub const DIZHI: [&str; 12] = ["子", "丑", "寅", "卯", "辰", "巳", "午", "未", "申", "酉", "戌", "亥"];

/// 生肖
pub const SHENGXIAO: [&str; 12] = ["鼠", "牛", "虎", "兔", "龙", "蛇", "马", "羊", "猴", "鸡", "狗", "猪"];

/// 天干五行 (阴阳属性)
pub const GAN_WUXING: [&str; 10] = ["阳木", "阴木", "阳火", "阴火", "阳土", "阴土", "阳金", "阴金", "阳水", "阴水"];

/// 地支五行 (阴阳属性)
pub const ZHI_WUXING: [&str; 12] = ["阳水", "阴土", "阳木", "阴木", "阳土", "阴火", "阳火", "阴土", "阳金", "阴金", "阳土", "阴水"];

/// 纳音 (60甲子)
pub const NAYIN: [&str; 30] = [
    "海中金", "炉中火", "大林木", "路旁土", "剑锋金", "山头火",
    "涧下水", "城头土", "白蜡金", "杨柳木", "泉中水", "屋上土",
    "霹雳火", "松柏木", "长流水", "砂石金", "山下火", "平地木",
    "壁上土", "金箔金", "覆灯火", "天河水", "大驿土", "钗钏金",
    "桑柘木", "大溪水", "沙中土", "天上火", "石榴木", "大海水"
];

/// 八字四柱
#[derive(Debug, Clone)]
pub struct BaziPillars {
    /// 年柱 (天干索引, 地支索引)
    pub year: (usize, usize),
    /// 月柱 (天干索引, 地支索引)
    pub month: (usize, usize),
    /// 日柱 (天干索引, 地支索引)
    pub day: (usize, usize),
    /// 时柱 (天干索引, 地支索引)
    pub hour: (usize, usize),
    /// 太阳黄经 (度)
    pub solar_longitude: f64,
    /// 当前节气
    pub solar_term: SolarTerm,
    /// 是否晚子时 (影响日柱)
    pub is_late_zi: bool,
}

/// 计算年柱
/// 
/// 以立春为界，立春前算上一年，立春后算当年。
/// 公式：year_gan = (year - 4) mod 10, year_zhi = (year - 4) mod 12
/// 
/// # 参数
/// - `year`: 公历年份
/// - `solar_longitude`: 太阳黄经
/// 
/// # 返回
/// - (天干索引, 地支索引)
pub fn calc_year_pillar(year: i32, solar_longitude: f64) -> (usize, usize) {
    // 立春 = 黄经 315°
    // 黄经在 [270°, 315°) 范围内为立春前，属上一年
    let bazi_year = if (270.0..315.0).contains(&solar_longitude) {
        year - 1
    } else {
        year
    };
    
    let gan_idx = ((bazi_year - 4) % 10 + 10) % 10;
    let zhi_idx = ((bazi_year - 4) % 12 + 12) % 12;
    
    (gan_idx as usize, zhi_idx as usize)
}

/// 计算月柱
/// 
/// 月支直接由太阳黄经决定：month_index = floor((λ - 315) / 30) mod 12
/// 月干由年干推算（五虎遁）。
/// 
/// # 五虎遁口诀
/// 甲己之年丙作首，乙庚之岁戊为头，
/// 丙辛必定寻庚起，丁壬壬位顺行流，
/// 若问戊癸何方发，甲寅之上好追求。
/// 
/// # 参数
/// - `year_gan_idx`: 年干索引
/// - `solar_longitude`: 太阳黄经
/// 
/// # 返回
/// - (天干索引, 地支索引)
pub fn calc_month_pillar(year_gan_idx: usize, solar_longitude: f64) -> (usize, usize) {
    // 月支: 直接用黄经计算
    // 315° = 寅月(2), 345° = 卯月(3), 15° = 辰月(4), ...
    // month_zhi_idx = floor((λ - 315) / 30) mod 12 + 2
    let adjusted_lon = (solar_longitude - 315.0 + 360.0).rem_euclid(360.0);
    let month_offset = (adjusted_lon / 30.0).floor() as usize;
    let month_zhi_idx = (month_offset + 2) % 12;  // 从寅(2)开始
    
    // 月干: 五虎遁
    // 正月(寅月)天干 = (年干 % 5) * 2 + 2
    let yin_month_gan = ((year_gan_idx % 5) * 2 + 2) % 10;
    
    // 当前月与寅月的偏移
    let month_gan_offset = (month_zhi_idx + 12 - 2) % 12;  // 从寅月算起的偏移
    let month_gan_idx = (yin_month_gan + month_gan_offset) % 10;
    
    (month_gan_idx, month_zhi_idx)
}

/// 计算日柱
/// 
/// 基准日: 1970-01-01 = 庚戌 (干支第47, 庚=6, 戌=10)
/// 公式: 从基准日算天数差，对60取模。
/// 
/// # 注意
/// 子时换日: 晚子时(23:00-24:00)日柱按次日计算。
/// 
/// # 参数
/// - `jd`: 儒略日
/// - `is_late_zi`: 是否晚子时
/// 
/// # 返回
/// - (天干索引, 地支索引)
pub fn calc_day_pillar(jd: f64, is_late_zi: bool) -> (usize, usize) {
    // 1970-01-01 00:00 UTC = JD 2440587.5
    // 该日为庚戌，庚=6, 戌=10
    // 干支60甲子中，庚戌 = 第47位 (0-indexed: 46)
    
    let days_from_epoch = (jd - 2440587.5).floor() as i32;
    
    // 晚子时按次日算
    let adjusted_days = if is_late_zi { days_from_epoch + 1 } else { days_from_epoch };
    
    // 天干: (6 + days) mod 10
    let gan_idx = ((adjusted_days + 6) % 10 + 10) % 10;
    // 地支: (10 + days) mod 12
    let zhi_idx = ((adjusted_days + 10) % 12 + 12) % 12;
    
    (gan_idx as usize, zhi_idx as usize)
}

/// 计算时柱
/// 
/// 时支由真太阳时确定，时干由日干推算（五鼠遁）。
/// 
/// # 五鼠遁口诀
/// 甲己还加甲，乙庚丙作初，
/// 丙辛从戊起，丁壬庚子居，
/// 戊癸何方发，壬子是真途。
/// 
/// # 参数
/// - `day_gan_idx`: 日干索引
/// - `hour_zhi_idx`: 时支索引 (0=子, 1=丑, ...)
/// 
/// # 返回
/// - (天干索引, 地支索引)
pub fn calc_hour_pillar(day_gan_idx: usize, hour_zhi_idx: usize) -> (usize, usize) {
    // 五鼠遁: 子时天干 = (日干 % 5) * 2
    let zi_hour_gan = (day_gan_idx % 5) * 2;
    
    // 当前时辰天干 = 子时天干 + 时支偏移
    let hour_gan_idx = (zi_hour_gan + hour_zhi_idx) % 10;
    
    (hour_gan_idx, hour_zhi_idx)
}

/// 计算八字四柱 (主入口)
/// 
/// # 参数
/// - `dt_utc`: UTC 时间
/// - `longitude`: 出生地经度 (东经为正)
/// 
/// # 返回
/// - `BaziPillars`: 四柱信息
pub fn calc_bazi_pillars(dt_utc: &DateTime<Utc>, longitude: f64) -> BaziPillars {
    let jd = datetime_to_jd(&dt_utc.naive_utc());
    let solar = solar_position(jd);
    let solar_longitude = solar.ecliptic_longitude;
    
    // 1. 计算真太阳时，确定时辰
    let tst_hour = true_solar_hour(dt_utc, longitude);
    let (hour_zhi_idx, is_late_zi) = hour_to_dizhi_index(tst_hour);
    
    // 2. 计算年柱 (以立春为界)
    let year = dt_utc.naive_utc().and_utc().year();
    let year_pillar = calc_year_pillar(year, solar_longitude);
    
    // 3. 计算月柱 (以节气为界)
    let month_pillar = calc_month_pillar(year_pillar.0, solar_longitude);
    
    // 4. 计算日柱 (注意晚子时换日)
    let day_pillar = calc_day_pillar(jd, is_late_zi);
    
    // 5. 计算时柱 (五鼠遁)
    let hour_pillar = calc_hour_pillar(day_pillar.0, hour_zhi_idx);
    
    let solar_term = SolarTerm::from_longitude(solar_longitude);
    
    BaziPillars {
        year: year_pillar,
        month: month_pillar,
        day: day_pillar,
        hour: hour_pillar,
        solar_longitude,
        solar_term,
        is_late_zi,
    }
}

/// 计算大运起运年龄
/// 
/// 算法来源: 《子平真诠》
/// - 阳年男命/阴年女命: 从出生日顺数到下一个"节"的天数
/// - 阴年男命/阳年女命: 从出生日逆数到上一个"节"的天数
/// - 每3天 = 1岁，余数按比例换算
/// 
/// # 参数
/// - `jd`: 出生时刻的儒略日
/// - `year_gan_idx`: 年干索引 (用于判断阴阳年)
/// - `is_male`: 是否男命
/// 
/// # 返回
/// - 起运年龄 (岁)
#[allow(clippy::manual_is_multiple_of)]
pub fn calc_dayun_start_age(jd: f64, year_gan_idx: usize, is_male: bool) -> f64 {
    let year_is_yang = year_gan_idx % 2 == 0;  // 阳年: 甲丙戊庚壬
    
    // 阳男阴女顺行，阴男阳女逆行
    let forward = (is_male && year_is_yang) || (!is_male && !year_is_yang);
    
    let days = if forward {
        let (next_jie_jd, _) = find_next_jie(jd);
        next_jie_jd - jd
    } else {
        let (prev_jie_jd, _) = find_prev_jie(jd);
        jd - prev_jie_jd
    };
    
    // 每3天 = 1岁
    let age = days / 3.0;
    
    // 至少1岁起运
    if age < 1.0 { 1.0 } else { age }
}

/// 获取纳音
/// 
/// # 参数
/// - `gan_idx`: 天干索引
/// - `zhi_idx`: 地支索引
/// 
/// # 返回
/// - 纳音名称
pub fn get_nayin(gan_idx: usize, zhi_idx: usize) -> &'static str {
    // 纳音按照干支组合的索引来查找
    // 甲子、乙丑 -> 海中金 (索引0)
    // 丙寅、丁卯 -> 炉中火 (索引1)
    // 每两个干支共享一个纳音
    let ganzhi_idx = (gan_idx * 12 + zhi_idx) % 60;
    let nayin_idx = ganzhi_idx / 2;
    NAYIN[nayin_idx % 30]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_year_pillar_after_lichun() {
        // 2025年2月5日 (立春后) 应为乙巳年
        let solar_lon = 316.0;  // 立春后
        let (gan, zhi) = calc_year_pillar(2025, solar_lon);
        assert_eq!(TIANGAN[gan], "乙", "立春后年干应为乙");
        assert_eq!(DIZHI[zhi], "巳", "立春后年支应为巳");
    }

    #[test]
    fn test_year_pillar_before_lichun() {
        // 2025年2月3日 (立春前) 应为甲辰年
        let solar_lon = 313.0;  // 立春前
        let (gan, zhi) = calc_year_pillar(2025, solar_lon);
        assert_eq!(TIANGAN[gan], "甲", "立春前年干应为甲");
        assert_eq!(DIZHI[zhi], "辰", "立春前年支应为辰");
    }

    #[test]
    fn test_month_pillar_formula() {
        // 立春(315°) = 寅月
        let (_, zhi) = calc_month_pillar(0, 315.0);
        assert_eq!(DIZHI[zhi], "寅", "315°应为寅月");
        
        // 惊蛰(345°) = 卯月
        let (_, zhi) = calc_month_pillar(0, 345.0);
        assert_eq!(DIZHI[zhi], "卯", "345°应为卯月");
        
        // 清明(15°) = 辰月
        let (_, zhi) = calc_month_pillar(0, 15.0);
        assert_eq!(DIZHI[zhi], "辰", "15°应为辰月");
    }

    #[test]
    fn test_five_tigers() {
        // 甲/己年正月 -> 丙寅
        let (gan, zhi) = calc_month_pillar(0, 315.0);  // 甲年，立春
        assert_eq!(TIANGAN[gan], "丙", "甲年正月天干应为丙");
        assert_eq!(DIZHI[zhi], "寅", "正月地支应为寅");
        
        // 乙/庚年正月 -> 戊寅
        let (gan, _) = calc_month_pillar(1, 315.0);  // 乙年
        assert_eq!(TIANGAN[gan], "戊", "乙年正月天干应为戊");
    }

    #[test]
    fn test_day_pillar_1970() {
        // 1970-01-01 = 庚戌
        let jd = 2440587.5;
        let (gan, zhi) = calc_day_pillar(jd, false);
        assert_eq!(TIANGAN[gan], "庚", "1970-01-01天干应为庚");
        assert_eq!(DIZHI[zhi], "戌", "1970-01-01地支应为戌");
    }

    #[test]
    fn test_five_rats() {
        // 甲日子时 -> 甲子
        let (gan, zhi) = calc_hour_pillar(0, 0);
        assert_eq!(TIANGAN[gan], "甲", "甲日子时天干应为甲");
        assert_eq!(DIZHI[zhi], "子", "子时地支应为子");
        
        // 乙日子时 -> 丙子
        let (gan, _) = calc_hour_pillar(1, 0);
        assert_eq!(TIANGAN[gan], "丙", "乙日子时天干应为丙");
        
        // 丙日子时 -> 戊子
        let (gan, _) = calc_hour_pillar(2, 0);
        assert_eq!(TIANGAN[gan], "戊", "丙日子时天干应为戊");
    }
}
