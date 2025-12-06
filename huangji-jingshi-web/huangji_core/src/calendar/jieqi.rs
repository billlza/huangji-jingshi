//! 二十四节气模块
//!
//! 节气以太阳黄经为准，每 15° 一个节气。
//! 八字排盘中，年以立春(315°)换年，月以"节"换月。
//!
//! 参考资料：
//! - 《天文年历》
//! - 中国科学院紫金山天文台

use crate::astro::solar::{solar_position, datetime_to_jd};
use chrono::NaiveDateTime;

/// 二十四节气枚举
/// 
/// 从春分(0°)开始，每 15° 一个节气
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum SolarTerm {
    /// 春分 0°
    Chunfen = 0,
    /// 清明 15°
    Qingming = 1,
    /// 谷雨 30°
    Guyu = 2,
    /// 立夏 45°
    Lixia = 3,
    /// 小满 60°
    Xiaoman = 4,
    /// 芒种 75°
    Mangzhong = 5,
    /// 夏至 90°
    Xiazhi = 6,
    /// 小暑 105°
    Xiaoshu = 7,
    /// 大暑 120°
    Dashu = 8,
    /// 立秋 135°
    Liqiu = 9,
    /// 处暑 150°
    Chushu = 10,
    /// 白露 165°
    Bailu = 11,
    /// 秋分 180°
    Qiufen = 12,
    /// 寒露 195°
    Hanlu = 13,
    /// 霜降 210°
    Shuangjiang = 14,
    /// 立冬 225°
    Lidong = 15,
    /// 小雪 240°
    Xiaoxue = 16,
    /// 大雪 255°
    Daxue = 17,
    /// 冬至 270°
    Dongzhi = 18,
    /// 小寒 285°
    Xiaohan = 19,
    /// 大寒 300°
    Dahan = 20,
    /// 立春 315°
    Lichun = 21,
    /// 雨水 330°
    Yushui = 22,
    /// 惊蛰 345°
    Jingzhe = 23,
}

impl SolarTerm {
    /// 节气中文名称
    pub fn name(&self) -> &'static str {
        match self {
            SolarTerm::Chunfen => "春分",
            SolarTerm::Qingming => "清明",
            SolarTerm::Guyu => "谷雨",
            SolarTerm::Lixia => "立夏",
            SolarTerm::Xiaoman => "小满",
            SolarTerm::Mangzhong => "芒种",
            SolarTerm::Xiazhi => "夏至",
            SolarTerm::Xiaoshu => "小暑",
            SolarTerm::Dashu => "大暑",
            SolarTerm::Liqiu => "立秋",
            SolarTerm::Chushu => "处暑",
            SolarTerm::Bailu => "白露",
            SolarTerm::Qiufen => "秋分",
            SolarTerm::Hanlu => "寒露",
            SolarTerm::Shuangjiang => "霜降",
            SolarTerm::Lidong => "立冬",
            SolarTerm::Xiaoxue => "小雪",
            SolarTerm::Daxue => "大雪",
            SolarTerm::Dongzhi => "冬至",
            SolarTerm::Xiaohan => "小寒",
            SolarTerm::Dahan => "大寒",
            SolarTerm::Lichun => "立春",
            SolarTerm::Yushui => "雨水",
            SolarTerm::Jingzhe => "惊蛰",
        }
    }
    
    /// 节气对应的太阳黄经度数
    pub fn longitude(&self) -> f64 {
        (*self as u8) as f64 * 15.0
    }
    
    /// 是否为"节"（用于月柱划分）
    /// 
    /// 节: 立春、惊蛰、清明、立夏、芒种、小暑、立秋、白露、寒露、立冬、大雪、小寒
    /// 气: 雨水、春分、谷雨、小满、夏至、大暑、处暑、秋分、霜降、小雪、冬至、大寒
    pub fn is_jie(&self) -> bool {
        matches!(self,
            SolarTerm::Lichun | SolarTerm::Jingzhe | SolarTerm::Qingming |
            SolarTerm::Lixia | SolarTerm::Mangzhong | SolarTerm::Xiaoshu |
            SolarTerm::Liqiu | SolarTerm::Bailu | SolarTerm::Hanlu |
            SolarTerm::Lidong | SolarTerm::Daxue | SolarTerm::Xiaohan
        )
    }
    
    /// 从索引创建节气
    pub fn from_index(idx: u8) -> Option<Self> {
        if idx < 24 {
            // Safe because we validate the range and SolarTerm is #[repr(u8)]
            Some(unsafe { std::mem::transmute::<u8, SolarTerm>(idx) })
        } else {
            None
        }
    }
    
    /// 从黄经度数获取当前所处的节气
    pub fn from_longitude(longitude: f64) -> Self {
        let normalized = longitude.rem_euclid(360.0);
        let idx = (normalized / 15.0).floor() as u8;
        Self::from_index(idx % 24).unwrap()
    }
}

/// 节气时刻点
#[derive(Debug, Clone)]
pub struct SolarTermPoint {
    /// 节气
    pub term: SolarTerm,
    /// 儒略日
    pub jd: f64,
    /// 太阳黄经度数
    pub longitude: f64,
}

/// 根据太阳黄经获取当前节气
pub fn current_solar_term(longitude: f64) -> SolarTerm {
    SolarTerm::from_longitude(longitude)
}

/// 计算某个节气的精确时刻 (使用二分法)
/// 
/// # 参数
/// - `year`: 公历年份
/// - `term`: 目标节气
/// 
/// # 返回
/// - 该节气的儒略日
pub fn find_solar_term_jd(year: i32, term: SolarTerm) -> f64 {
    let target_lon = term.longitude();
    
    // 估算起始搜索点
    // 立春约在2月4日，索引21，每个节气约15天
    let term_idx = term as u8;
    let days_from_lichun = if term_idx >= 21 {
        (term_idx - 21) as f64 * 15.22
    } else {
        (term_idx as f64 + 3.0) * 15.22
    };
    
    // 该年立春约在2月4日，JD约 = 年初 + 35天
    let year_start_jd = datetime_to_jd(&NaiveDateTime::parse_from_str(
        &format!("{}-01-01 00:00:00", year), "%Y-%m-%d %H:%M:%S"
    ).unwrap());
    
    let mut jd_low = year_start_jd + days_from_lichun - 20.0;
    let mut jd_high = year_start_jd + days_from_lichun + 20.0;
    
    // 二分法查找
    for _ in 0..50 {
        let jd_mid = (jd_low + jd_high) / 2.0;
        let lon = solar_position(jd_mid).ecliptic_longitude;
        
        // 处理黄经跨越 0°/360° 的情况
        let diff = (lon - target_lon).rem_euclid(360.0);
        let diff = if diff > 180.0 { diff - 360.0 } else { diff };
        
        if diff.abs() < 0.0001 {
            return jd_mid;
        }
        
        if diff > 0.0 {
            jd_high = jd_mid;
        } else {
            jd_low = jd_mid;
        }
    }
    
    (jd_low + jd_high) / 2.0
}

/// 查找某年的所有节气时刻
pub fn find_solar_terms_for_year(year: i32) -> Vec<SolarTermPoint> {
    let mut terms = Vec::with_capacity(24);
    
    for i in 0..24 {
        if let Some(term) = SolarTerm::from_index(i) {
            let jd = find_solar_term_jd(year, term);
            terms.push(SolarTermPoint {
                term,
                jd,
                longitude: term.longitude(),
            });
        }
    }
    
    // 按儒略日排序
    terms.sort_by(|a, b| a.jd.partial_cmp(&b.jd).unwrap());
    
    terms
}

/// 查找给定时刻之后的下一个"节"
/// 
/// # 参数
/// - `jd`: 当前儒略日
/// 
/// # 返回
/// - (下一个节的儒略日, 节气)
pub fn find_next_jie(jd: f64) -> (f64, SolarTerm) {
    let current_lon = solar_position(jd).ecliptic_longitude;
    let current_term = SolarTerm::from_longitude(current_lon);
    
    // 找下一个"节"
    let mut next_idx = (current_term as u8 + 1) % 24;
    loop {
        let term = SolarTerm::from_index(next_idx).unwrap();
        if term.is_jie() {
            // 计算这个节的精确时刻
            let target_lon = term.longitude();
            let lon_diff = (target_lon - current_lon).rem_euclid(360.0);
            let approx_days = lon_diff / 0.9856; // 太阳每天移动约 0.9856°
            
            let approx_jd = jd + approx_days;
            
            // 使用二分法精确查找
            let mut jd_low = approx_jd - 5.0;
            let mut jd_high = approx_jd + 5.0;
            
            for _ in 0..30 {
                let jd_mid = (jd_low + jd_high) / 2.0;
                let lon = solar_position(jd_mid).ecliptic_longitude;
                let diff = (lon - target_lon).rem_euclid(360.0);
                let diff = if diff > 180.0 { diff - 360.0 } else { diff };
                
                if diff.abs() < 0.0001 {
                    return (jd_mid, term);
                }
                
                if diff > 0.0 {
                    jd_high = jd_mid;
                } else {
                    jd_low = jd_mid;
                }
            }
            
            return ((jd_low + jd_high) / 2.0, term);
        }
        next_idx = (next_idx + 1) % 24;
    }
}

/// 查找给定时刻之前的上一个"节"
/// 
/// # 参数
/// - `jd`: 当前儒略日
/// 
/// # 返回
/// - (上一个节的儒略日, 节气)
pub fn find_prev_jie(jd: f64) -> (f64, SolarTerm) {
    let current_lon = solar_position(jd).ecliptic_longitude;
    
    // 找上一个"节"
    let mut prev_idx = ((SolarTerm::from_longitude(current_lon) as i8 - 1 + 24) % 24) as u8;
    loop {
        let term = SolarTerm::from_index(prev_idx).unwrap();
        if term.is_jie() {
            let target_lon = term.longitude();
            let lon_diff = (current_lon - target_lon).rem_euclid(360.0);
            let approx_days = lon_diff / 0.9856;
            
            let approx_jd = jd - approx_days;
            
            let mut jd_low = approx_jd - 5.0;
            let mut jd_high = approx_jd + 5.0;
            
            for _ in 0..30 {
                let jd_mid = (jd_low + jd_high) / 2.0;
                let lon = solar_position(jd_mid).ecliptic_longitude;
                let diff = (lon - target_lon).rem_euclid(360.0);
                let diff = if diff > 180.0 { diff - 360.0 } else { diff };
                
                if diff.abs() < 0.0001 {
                    return (jd_mid, term);
                }
                
                if diff > 0.0 {
                    jd_high = jd_mid;
                } else {
                    jd_low = jd_mid;
                }
            }
            
            return ((jd_low + jd_high) / 2.0, term);
        }
        prev_idx = ((prev_idx as i8 - 1 + 24) % 24) as u8;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_solar_term_longitude() {
        assert_eq!(SolarTerm::Chunfen.longitude(), 0.0);
        assert_eq!(SolarTerm::Xiazhi.longitude(), 90.0);
        assert_eq!(SolarTerm::Qiufen.longitude(), 180.0);
        assert_eq!(SolarTerm::Dongzhi.longitude(), 270.0);
        assert_eq!(SolarTerm::Lichun.longitude(), 315.0);
    }

    #[test]
    fn test_is_jie() {
        assert!(SolarTerm::Lichun.is_jie());
        assert!(SolarTerm::Jingzhe.is_jie());
        assert!(!SolarTerm::Yushui.is_jie());  // 雨水是气，不是节
        assert!(!SolarTerm::Chunfen.is_jie()); // 春分是气
    }

    #[test]
    fn test_from_longitude() {
        assert_eq!(SolarTerm::from_longitude(315.0), SolarTerm::Lichun);
        assert_eq!(SolarTerm::from_longitude(316.0), SolarTerm::Lichun);
        assert_eq!(SolarTerm::from_longitude(329.9), SolarTerm::Lichun);
        assert_eq!(SolarTerm::from_longitude(330.0), SolarTerm::Yushui);
    }
}
