//! 太阳位置与时间计算
//!
//! 参考资料：
//! - Jean Meeus, "Astronomical Algorithms" (2nd ed.)
//! - NOAA Solar Calculator: https://gml.noaa.gov/grad/solcalc/
//! - Equation of Time: https://www.sws.bom.gov.au/Category/Educational/The%20Sun%20and%20Solar%20Activity/General%20Info/EquationOfTime.pdf

use chrono::{DateTime, NaiveDateTime, Timelike, Utc};

/// 太阳位置信息
#[derive(Debug, Clone, Copy)]
pub struct SolarPosition {
    /// 太阳黄经 (度, 0-360)
    pub ecliptic_longitude: f64,
    /// 均时差 (分钟)
    pub equation_of_time: f64,
}

/// 将 NaiveDateTime 转换为儒略日 (Julian Day)
/// 
/// JD 2440587.5 = 1970-01-01 00:00:00 UTC
pub fn datetime_to_jd(dt: &NaiveDateTime) -> f64 {
    let timestamp = dt.and_utc().timestamp();
    (timestamp as f64 / 86400.0) + 2440587.5
}

/// 将 DateTime<Utc> 转换为儒略日
pub fn utc_to_jd(dt: &DateTime<Utc>) -> f64 {
    let timestamp = dt.timestamp();
    (timestamp as f64 / 86400.0) + 2440587.5
}

/// 计算太阳位置（黄经和均时差）
/// 
/// 使用 VSOP87 简化算法，精度约 0.01°，足够八字排盘使用。
/// 
/// # 参数
/// - `jd`: 儒略日
/// 
/// # 返回
/// - `SolarPosition`: 太阳黄经(度)和均时差(分钟)
pub fn solar_position(jd: f64) -> SolarPosition {
    // 儒略世纪数 (从 J2000.0 = JD 2451545.0 起算)
    let t = (jd - 2451545.0) / 36525.0;
    
    // ========== 太阳黄经计算 (VSOP87 简化) ==========
    
    // 太阳平黄经 (Mean Longitude)
    let l0 = (280.46646 + 36000.76983 * t + 0.0003032 * t * t).rem_euclid(360.0);
    
    // 太阳平近点角 (Mean Anomaly)
    let m = (357.52911 + 35999.05029 * t - 0.0001537 * t * t).rem_euclid(360.0);
    let m_rad = m.to_radians();
    
    // 地球轨道偏心率 (保留用于更精确计算)
    let _e = 0.016708634 - 0.000042037 * t - 0.0000001267 * t * t;
    
    // 太阳中心差 (Equation of Center)
    let c = (1.914602 - 0.004817 * t - 0.000014 * t * t) * m_rad.sin()
          + (0.019993 - 0.000101 * t) * (2.0 * m_rad).sin()
          + 0.000289 * (3.0 * m_rad).sin();
    
    // 太阳真黄经 (True Longitude)
    let sun_lon = (l0 + c).rem_euclid(360.0);
    
    // ========== 均时差计算 (Equation of Time) ==========
    
    // 黄赤交角 (Obliquity of Ecliptic)
    let epsilon = 23.439291 - 0.0130042 * t;
    let epsilon_rad = epsilon.to_radians();
    
    // 太阳赤经 (Right Ascension)
    let y = epsilon_rad.cos() * sun_lon.to_radians().sin();
    let x = sun_lon.to_radians().cos();
    let alpha = y.atan2(x).to_degrees().rem_euclid(360.0);
    
    // 均时差 = 平黄经 - 赤经 (转换为分钟)
    let mut eot = l0 - alpha;
    if eot > 180.0 { eot -= 360.0; }
    if eot < -180.0 { eot += 360.0; }
    let eot_minutes = eot * 4.0; // 1度 = 4分钟
    
    SolarPosition {
        ecliptic_longitude: sun_lon,
        equation_of_time: eot_minutes,
    }
}

/// 简化的均时差计算 (使用 Spencer 公式)
/// 
/// 公式来源: Spencer (1971), "Fourier series representation of the position of the sun"
/// 精度约 30 秒，适合快速计算。
/// 
/// # 参数
/// - `day_of_year`: 一年中的第几天 (1-366)
/// 
/// # 返回
/// - 均时差 (分钟)
pub fn equation_of_time_spencer(day_of_year: u32) -> f64 {
    // B = 360 * (N - 81) / 365
    let b = (360.0 * (day_of_year as f64 - 81.0) / 365.0).to_radians();
    
    // EoT = 9.87 * sin(2B) - 7.67 * sin(B + 78.7°)
    9.87 * (2.0 * b).sin() - 7.67 * (b + 78.7_f64.to_radians()).sin()
}

/// 计算真太阳时
/// 
/// 真太阳时 = 标准时 + 经度修正 + 均时差
/// 
/// # 参数
/// - `dt_utc`: UTC 时间
/// - `longitude`: 出生地经度 (东经为正)
/// - `standard_meridian`: 标准时区中央经线 (如北京时间用 120.0)
/// 
/// # 返回
/// - 真太阳时 (NaiveDateTime)
pub fn true_solar_time(
    dt_utc: &DateTime<Utc>,
    longitude: f64,
    standard_meridian: f64,
) -> NaiveDateTime {
    let jd = utc_to_jd(dt_utc);
    let solar = solar_position(jd);
    
    // 经度修正: 每度 = 4分钟
    // 东边比标准时区快，西边慢
    let longitude_correction_minutes = (longitude - standard_meridian) * 4.0;
    
    // 总修正量 (分钟) = 经度修正 + 均时差
    let total_correction_minutes = longitude_correction_minutes + solar.equation_of_time;
    
    // 转换为秒并应用
    let correction_seconds = (total_correction_minutes * 60.0).round() as i64;
    
    // 先转到标准时区时间，再加修正
    let local_standard = *dt_utc + chrono::Duration::hours((standard_meridian / 15.0).round() as i64);
    let true_solar = local_standard + chrono::Duration::seconds(correction_seconds);
    
    true_solar.naive_utc()
}

/// 计算真太阳时（简化版，直接返回小时数）
/// 
/// # 参数
/// - `dt_utc`: UTC 时间
/// - `longitude`: 出生地经度 (东经为正)
/// 
/// # 返回
/// - 真太阳时小时数 (0.0 - 24.0)
pub fn true_solar_hour(dt_utc: &DateTime<Utc>, longitude: f64) -> f64 {
    let jd = utc_to_jd(dt_utc);
    let solar = solar_position(jd);
    
    // UTC 时间转为小时
    let utc_hour = dt_utc.hour() as f64 
                 + dt_utc.minute() as f64 / 60.0 
                 + dt_utc.second() as f64 / 3600.0;
    
    // 经度时差: 每15度 = 1小时
    let longitude_hours = longitude / 15.0;
    
    // 均时差转小时
    let eot_hours = solar.equation_of_time / 60.0;
    
    // 真太阳时 = UTC + 经度时差 + 均时差
    let tst = utc_hour + longitude_hours + eot_hours;
    
    // 归一化到 0-24
    tst.rem_euclid(24.0)
}

/// 根据真太阳时小时数获取时辰地支索引
/// 
/// 子时: 23:00-01:00 (索引 0)
/// 丑时: 01:00-03:00 (索引 1)
/// ...
/// 亥时: 21:00-23:00 (索引 11)
/// 
/// # 参数
/// - `true_solar_hour`: 真太阳时小时数 (0.0 - 24.0)
/// 
/// # 返回
/// - 时辰地支索引 (0=子, 1=丑, ..., 11=亥)
/// - 是否为晚子时 (23:00-24:00)
pub fn hour_to_dizhi_index(true_solar_hour: f64) -> (usize, bool) {
    // 晚子时 (23:00-24:00): 属于子时，但日柱要按次日算
    if true_solar_hour >= 23.0 {
        return (0, true); // 子时，晚子时
    }
    
    // 早子时 (00:00-01:00): 属于子时，日柱按当日算
    if true_solar_hour < 1.0 {
        return (0, false); // 子时，早子时
    }
    
    // 其他时辰: (hour + 1) / 2
    let index = ((true_solar_hour + 1.0) / 2.0).floor() as usize;
    (index % 12, false)
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;

    #[test]
    fn test_solar_longitude_vernal_equinox() {
        // 2024年春分: 3月20日 约 03:06 UTC
        // 太阳黄经应接近 0°
        let dt = Utc.with_ymd_and_hms(2024, 3, 20, 3, 6, 0).unwrap();
        let jd = utc_to_jd(&dt);
        let solar = solar_position(jd);
        println!("春分太阳黄经: {:.2}°", solar.ecliptic_longitude);
        assert!(solar.ecliptic_longitude < 1.0 || solar.ecliptic_longitude > 359.0);
    }

    #[test]
    fn test_solar_longitude_lichun() {
        // 2025年立春: 2月3日 约 22:10 UTC (北京时间 2月4日 06:10)
        // 太阳黄经应接近 315°
        let dt = Utc.with_ymd_and_hms(2025, 2, 3, 22, 10, 0).unwrap();
        let jd = utc_to_jd(&dt);
        let solar = solar_position(jd);
        println!("立春太阳黄经: {:.2}°", solar.ecliptic_longitude);
        assert!((solar.ecliptic_longitude - 315.0).abs() < 1.0);
    }

    #[test]
    fn test_equation_of_time() {
        // 均时差在 ±16 分钟范围内
        for day in [1, 45, 105, 166, 227, 288, 349] {
            let eot = equation_of_time_spencer(day);
            println!("第{}天均时差: {:.2}分钟", day, eot);
            assert!(eot.abs() < 20.0);
        }
    }

    #[test]
    fn test_hour_to_dizhi() {
        // 子时 (23:00-01:00)
        assert_eq!(hour_to_dizhi_index(23.5), (0, true));  // 晚子时
        assert_eq!(hour_to_dizhi_index(0.5), (0, false));  // 早子时
        
        // 丑时 (01:00-03:00)
        assert_eq!(hour_to_dizhi_index(1.5), (1, false));
        assert_eq!(hour_to_dizhi_index(2.5), (1, false));
        
        // 午时 (11:00-13:00)
        assert_eq!(hour_to_dizhi_index(12.0), (6, false));
    }
}
