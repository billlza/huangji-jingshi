use chrono::{TimeZone, Utc};
use huangji_core::lunar;

fn main() {
    // 默认以北京时间 + 东经116.4（北京）做“本地年月日/时”推导
    let tz_offset_minutes: i32 = 480;
    let lon: f64 = 116.4;
    let use_true_solar_time: bool = false;

    // Test Case 1: 2025-11-29 (Today's date in <env>)
    // Expected: 乙巳年 丁亥月 壬寅日
    let dt1 = Utc.with_ymd_and_hms(2025, 11, 29, 12, 0, 0).unwrap();
    let res1 = lunar::compute_lunar(&dt1, tz_offset_minutes, lon, use_true_solar_time).unwrap();
    println!("2025-11-29: {} {} {}", res1.ganzhi_year, res1.ganzhi_month, res1.ganzhi_day);
    assert_eq!(res1.ganzhi_day, "壬寅");
    assert_eq!(res1.ganzhi_month, "丁亥");
    assert_eq!(res1.ganzhi_year, "乙巳");

    // Test Case 2: 2024-02-03 (Before LiChun)
    // LiChun is 2024-02-04. So 2024-02-03 should be GuiMao (Rabbit), not JiaChen (Dragon).
    let dt2 = Utc.with_ymd_and_hms(2024, 2, 3, 12, 0, 0).unwrap();
    let res2 = lunar::compute_lunar(&dt2, tz_offset_minutes, lon, use_true_solar_time).unwrap();
    println!("2024-02-03: {} (Expected: 癸卯)", res2.ganzhi_year);
    assert_eq!(res2.ganzhi_year, "癸卯");

    // Test Case 3: 2024-02-05 (After LiChun)
    let dt3 = Utc.with_ymd_and_hms(2024, 2, 5, 12, 0, 0).unwrap();
    let res3 = lunar::compute_lunar(&dt3, tz_offset_minutes, lon, use_true_solar_time).unwrap();
    println!("2024-02-05: {} (Expected: 甲辰)", res3.ganzhi_year);
    assert_eq!(res3.ganzhi_year, "甲辰");
    
    println!("All tests passed!");
}
