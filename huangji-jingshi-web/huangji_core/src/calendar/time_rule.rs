//! 时间规则模块
//! 
//! 提供 UTC 时间到规则时间（rule_dt）的转换，以及规则时间到经世年（hj_year）的转换。
//! 
//! 符号约定：
//! - tzOffsetMinutes: 时区偏移（分钟），东为正 UTC+8=+480, 西为负 UTC-5=-300
//! - 注意：与 JS Date.getTimezoneOffset() 符号相反！

use chrono::{DateTime, Utc, FixedOffset, Datelike, Duration};

/// 岁首模式
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum YearStartMode {
    /// 公历岁首模式：以公历 1 月 1 日为年份切换点
    GregorianNewYear,
    /// 立春岁首模式：以立春节气时刻为年份切换点（预留）
    Lichun,
}

impl Default for YearStartMode {
    fn default() -> Self {
        YearStartMode::GregorianNewYear
    }
}

/// 将 UTC 时间转换为规则时间（rule_dt）
/// 
/// # Arguments
/// * `utc` - UTC 时间
/// * `tz_offset_minutes` - 时区偏移（分钟），东为正 UTC+8=+480, 西为负 UTC-5=-300
/// * `lon` - 经度（用于真太阳时校正）
/// * `use_true_solar_time` - 是否使用真太阳时
/// 
/// # Returns
/// 规则时间（带固定时区偏移）
/// 
/// # 真太阳时校正
/// 以所选时区中央经线为基准：
/// - `tz_offset_hours = tz_offset_minutes as f64 / 60.0`（支持非整小时时区如 +5:30）
/// - `central_meridian = 15.0 * tz_offset_hours`
/// - `delta_minutes = 4.0 * (lon - central_meridian)`
pub fn to_rule_datetime(
    utc: DateTime<Utc>,
    tz_offset_minutes: i32,
    lon: f64,
    use_true_solar_time: bool,
) -> DateTime<FixedOffset> {
    // 创建固定偏移时区
    let offset = FixedOffset::east_opt(tz_offset_minutes * 60)
        .unwrap_or_else(|| FixedOffset::east_opt(8 * 3600).unwrap()); // 默认 UTC+8
    
    // 转换为本地时间
    let local_dt = utc.with_timezone(&offset);
    
    if use_true_solar_time {
        // 真太阳时校正
        // 使用浮点除法支持非整小时时区（如 +5:30 = +330 分钟）
        let tz_offset_hours: f64 = tz_offset_minutes as f64 / 60.0;
        let central_meridian: f64 = 15.0 * tz_offset_hours;
        let delta_minutes: f64 = 4.0 * (lon - central_meridian);
        
        // 应用真太阳时校正
        let delta_seconds = (delta_minutes * 60.0).round() as i64;
        local_dt + Duration::seconds(delta_seconds)
    } else {
        local_dt
    }
}

/// 将规则时间转换为经世年（hj_year）
/// 
/// # Arguments
/// * `rule_dt` - 规则时间
/// * `mode` - 岁首模式
/// 
/// # Returns
/// 经世年（历史纪年，无公元 0 年）
/// 
/// # 注意
/// - 公历岁首模式：直接使用公历年份
/// - 立春岁首模式：根据立春时刻判断年份归属（预留）
pub fn datetime_to_hj_year(
    rule_dt: DateTime<FixedOffset>,
    mode: YearStartMode,
) -> i32 {
    match mode {
        YearStartMode::GregorianNewYear => {
            // 公历岁首模式：直接使用公历年份
            let year = rule_dt.year();
            // 公历年份直接对应经世年（公历也没有 0 年）
            // 但 chrono 使用天文年份（有 0 年），需要转换
            // chrono: ..., -2, -1, 0, 1, 2, ...
            // 历史纪年: ..., -2, -1, 1, 2, ... (无 0 年)
            if year <= 0 {
                year - 1  // 0 -> -1, -1 -> -2, etc.
            } else {
                year
            }
        }
        YearStartMode::Lichun => {
            // 立春岁首模式（预留）
            // TODO: 根据立春时刻判断年份归属
            // 暂时使用公历岁首模式
            let year = rule_dt.year();
            if year <= 0 {
                year - 1
            } else {
                year
            }
        }
    }
}

/// 便捷函数：从 UTC 时间直接获取经世年
/// 
/// # Arguments
/// * `utc` - UTC 时间
/// * `tz_offset_minutes` - 时区偏移（分钟），东为正 UTC+8=+480, 西为负 UTC-5=-300
/// * `lon` - 经度
/// * `use_true_solar_time` - 是否使用真太阳时
/// * `mode` - 岁首模式
/// 
/// # Returns
/// 经世年（历史纪年，无公元 0 年）
pub fn utc_to_hj_year(
    utc: DateTime<Utc>,
    tz_offset_minutes: i32,
    lon: f64,
    use_true_solar_time: bool,
    mode: YearStartMode,
) -> i32 {
    let rule_dt = to_rule_datetime(utc, tz_offset_minutes, lon, use_true_solar_time);
    datetime_to_hj_year(rule_dt, mode)
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;
    use chrono::Timelike;
    
    #[test]
    fn test_to_rule_datetime_utc8() {
        // UTC 2025-12-18T13:48:00Z + UTC+8 = 2025-12-18T21:48:00+08:00
        let utc = Utc.with_ymd_and_hms(2025, 12, 18, 13, 48, 0).unwrap();
        let rule_dt = to_rule_datetime(utc, 480, 116.4, false);
        
        assert_eq!(rule_dt.year(), 2025);
        assert_eq!(rule_dt.month(), 12);
        assert_eq!(rule_dt.day(), 18);
        assert_eq!(rule_dt.hour(), 21);
        assert_eq!(rule_dt.minute(), 48);
    }
    
    #[test]
    fn test_to_rule_datetime_utc_minus_5() {
        // UTC 2025-12-18T13:00:00Z + UTC-5 = 2025-12-18T08:00:00-05:00
        let utc = Utc.with_ymd_and_hms(2025, 12, 18, 13, 0, 0).unwrap();
        let rule_dt = to_rule_datetime(utc, -300, -75.0, false);
        
        assert_eq!(rule_dt.year(), 2025);
        assert_eq!(rule_dt.month(), 12);
        assert_eq!(rule_dt.day(), 18);
        assert_eq!(rule_dt.hour(), 8);
        assert_eq!(rule_dt.minute(), 0);
    }
    
    #[test]
    fn test_datetime_to_hj_year_gregorian() {
        let offset = FixedOffset::east_opt(8 * 3600).unwrap();
        
        // 2025 年
        let dt = offset.with_ymd_and_hms(2025, 6, 15, 12, 0, 0).unwrap();
        assert_eq!(datetime_to_hj_year(dt, YearStartMode::GregorianNewYear), 2025);
        
        // 1 AD
        let dt = offset.with_ymd_and_hms(1, 6, 15, 12, 0, 0).unwrap();
        assert_eq!(datetime_to_hj_year(dt, YearStartMode::GregorianNewYear), 1);
        
        // 1 BC (chrono year 0)
        let dt = offset.with_ymd_and_hms(0, 6, 15, 12, 0, 0).unwrap();
        assert_eq!(datetime_to_hj_year(dt, YearStartMode::GregorianNewYear), -1);
        
        // 2 BC (chrono year -1)
        let dt = offset.with_ymd_and_hms(-1, 6, 15, 12, 0, 0).unwrap();
        assert_eq!(datetime_to_hj_year(dt, YearStartMode::GregorianNewYear), -2);
    }
    
    #[test]
    fn test_true_solar_time_correction() {
        // 北京时间 (UTC+8)，北京经度 116.4°E
        // 中央经线 = 15 * 8 = 120°E
        // delta = 4 * (116.4 - 120) = -14.4 分钟
        let utc = Utc.with_ymd_and_hms(2025, 12, 18, 12, 0, 0).unwrap();
        let rule_dt = to_rule_datetime(utc, 480, 116.4, true);
        
        // 本地时间 20:00，真太阳时校正 -14.4 分钟 ≈ 19:45:36
        assert_eq!(rule_dt.hour(), 19);
        assert!(rule_dt.minute() < 50); // 应该在 45-46 分钟左右
    }
    
    #[test]
    fn test_non_integer_hour_timezone() {
        // 印度时区 UTC+5:30 = +330 分钟
        // 中央经线 = 15 * 5.5 = 82.5°E
        let utc = Utc.with_ymd_and_hms(2025, 12, 18, 12, 0, 0).unwrap();
        let rule_dt = to_rule_datetime(utc, 330, 82.5, false);
        
        // UTC 12:00 + 5:30 = 17:30
        assert_eq!(rule_dt.hour(), 17);
        assert_eq!(rule_dt.minute(), 30);
    }
}
