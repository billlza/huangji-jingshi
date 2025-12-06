//! 八字排盘自动化测试
//! 
//! 使用 bazi_fixtures.json 中的测试用例验证排盘算法正确性。
//! 
//! 测试用例来源：
//! - 已知的命盘实例
//! - 边界条件（立春换年、节气换月、晚子时换日）
//! - 五虎遁、五鼠遁验证

use chrono::{TimeZone, Utc};
use huangji_core::calendar::ganzhi::{calc_bazi_pillars, TIANGAN, DIZHI};

#[derive(Debug, serde::Deserialize)]
struct TestCase {
    name: String,
    description: String,
    datetime_utc: String,
    #[allow(dead_code)]
    timezone: i32,
    longitude: f64,
    #[allow(dead_code)]
    latitude: f64,
    #[allow(dead_code)]
    gender: String,
    expected: Expected,
}

#[derive(Debug, serde::Deserialize)]
struct Expected {
    year_gan: Option<String>,
    year_zhi: Option<String>,
    month_gan: Option<String>,
    month_zhi: Option<String>,
    day_gan: Option<String>,
    day_zhi: Option<String>,
    hour_gan: Option<String>,
    hour_zhi: Option<String>,
    is_late_zi: Option<bool>,
    #[allow(dead_code)]
    note: Option<String>,
}

fn load_test_cases() -> Vec<TestCase> {
    let json_str = include_str!("bazi_fixtures.json");
    serde_json::from_str(json_str).expect("Failed to parse bazi_fixtures.json")
}

#[test]
fn test_bazi_fixtures() {
    let cases = load_test_cases();
    
    for case in cases {
        println!("\n=== 测试用例: {} ===", case.name);
        println!("描述: {}", case.description);
        
        // 解析时间
        let dt = chrono::DateTime::parse_from_rfc3339(&case.datetime_utc)
            .map(|dt| dt.with_timezone(&Utc))
            .expect(&format!("无法解析时间: {}", case.datetime_utc));
        
        // 计算八字
        let pillars = calc_bazi_pillars(&dt, case.longitude);
        
        println!("计算结果: {}{} {}{} {}{} {}{}",
            TIANGAN[pillars.year.0], DIZHI[pillars.year.1],
            TIANGAN[pillars.month.0], DIZHI[pillars.month.1],
            TIANGAN[pillars.day.0], DIZHI[pillars.day.1],
            TIANGAN[pillars.hour.0], DIZHI[pillars.hour.1]
        );
        println!("太阳黄经: {:.2}°, 节气: {}, 晚子时: {}",
            pillars.solar_longitude, pillars.solar_term.name(), pillars.is_late_zi
        );
        
        // 验证年柱
        if let Some(ref expected_gan) = case.expected.year_gan {
            let actual_gan = TIANGAN[pillars.year.0];
            assert_eq!(
                actual_gan, expected_gan,
                "[{}] 年干不匹配: 期望 {}, 实际 {}", case.name, expected_gan, actual_gan
            );
        }
        if let Some(ref expected_zhi) = case.expected.year_zhi {
            let actual_zhi = DIZHI[pillars.year.1];
            assert_eq!(
                actual_zhi, expected_zhi,
                "[{}] 年支不匹配: 期望 {}, 实际 {}", case.name, expected_zhi, actual_zhi
            );
        }
        
        // 验证月柱
        if let Some(ref expected_gan) = case.expected.month_gan {
            let actual_gan = TIANGAN[pillars.month.0];
            assert_eq!(
                actual_gan, expected_gan,
                "[{}] 月干不匹配: 期望 {}, 实际 {}", case.name, expected_gan, actual_gan
            );
        }
        if let Some(ref expected_zhi) = case.expected.month_zhi {
            let actual_zhi = DIZHI[pillars.month.1];
            assert_eq!(
                actual_zhi, expected_zhi,
                "[{}] 月支不匹配: 期望 {}, 实际 {}", case.name, expected_zhi, actual_zhi
            );
        }
        
        // 验证日柱
        if let Some(ref expected_gan) = case.expected.day_gan {
            let actual_gan = TIANGAN[pillars.day.0];
            assert_eq!(
                actual_gan, expected_gan,
                "[{}] 日干不匹配: 期望 {}, 实际 {}", case.name, expected_gan, actual_gan
            );
        }
        if let Some(ref expected_zhi) = case.expected.day_zhi {
            let actual_zhi = DIZHI[pillars.day.1];
            assert_eq!(
                actual_zhi, expected_zhi,
                "[{}] 日支不匹配: 期望 {}, 实际 {}", case.name, expected_zhi, actual_zhi
            );
        }
        
        // 验证时柱
        if let Some(ref expected_gan) = case.expected.hour_gan {
            let actual_gan = TIANGAN[pillars.hour.0];
            assert_eq!(
                actual_gan, expected_gan,
                "[{}] 时干不匹配: 期望 {}, 实际 {}", case.name, expected_gan, actual_gan
            );
        }
        if let Some(ref expected_zhi) = case.expected.hour_zhi {
            let actual_zhi = DIZHI[pillars.hour.1];
            assert_eq!(
                actual_zhi, expected_zhi,
                "[{}] 时支不匹配: 期望 {}, 实际 {}", case.name, expected_zhi, actual_zhi
            );
        }
        
        // 验证晚子时标志
        if let Some(expected_late_zi) = case.expected.is_late_zi {
            assert_eq!(
                pillars.is_late_zi, expected_late_zi,
                "[{}] 晚子时标志不匹配: 期望 {}, 实际 {}", case.name, expected_late_zi, pillars.is_late_zi
            );
        }
        
        println!("✓ 通过");
    }
}

/// 边界一致性测试：确保节气、年柱、月柱不互相矛盾
#[test]
fn test_boundary_consistency() {
    // 测试立春前后的边界
    let before_lichun = Utc.with_ymd_and_hms(2025, 2, 3, 12, 0, 0).unwrap();
    let after_lichun = Utc.with_ymd_and_hms(2025, 2, 5, 12, 0, 0).unwrap();
    
    let pillars_before = calc_bazi_pillars(&before_lichun, 116.4);
    let pillars_after = calc_bazi_pillars(&after_lichun, 116.4);
    
    println!("立春前: {}{} {}{} 黄经: {:.2}°", 
        TIANGAN[pillars_before.year.0], DIZHI[pillars_before.year.1],
        TIANGAN[pillars_before.month.0], DIZHI[pillars_before.month.1],
        pillars_before.solar_longitude
    );
    println!("立春后: {}{} {}{} 黄经: {:.2}°",
        TIANGAN[pillars_after.year.0], DIZHI[pillars_after.year.1],
        TIANGAN[pillars_after.month.0], DIZHI[pillars_after.month.1],
        pillars_after.solar_longitude
    );
    
    // 立春前黄经应 < 315
    assert!(pillars_before.solar_longitude < 315.0 && pillars_before.solar_longitude >= 270.0,
        "立春前黄经应在 [270, 315) 范围内，实际: {:.2}°", pillars_before.solar_longitude);
    
    // 立春后黄经应 >= 315 或 < 270 (跨过0度)
    assert!(pillars_after.solar_longitude >= 315.0 || pillars_after.solar_longitude < 270.0,
        "立春后黄经应 >= 315 或 < 270，实际: {:.2}°", pillars_after.solar_longitude);
    
    // 立春前后年支应该相差1
    assert_ne!(pillars_before.year.1, pillars_after.year.1,
        "立春前后年支应该不同");
    
    // 立春后月支应为寅(2)
    assert_eq!(DIZHI[pillars_after.month.1], "寅",
        "立春后月支应为寅月");
    
    // 立春前月支应为丑(1)
    assert_eq!(DIZHI[pillars_before.month.1], "丑",
        "立春前月支应为丑月");
}

/// 测试一年中的12个月是否正确切换
#[test]
fn test_month_transitions() {
    // 每个月中旬的日期，检查月支是否正确
    let test_dates = [
        ("2025-02-15", "寅"),  // 立春后
        ("2025-03-15", "卯"),  // 惊蛰后
        ("2025-04-15", "辰"),  // 清明后
        ("2025-05-15", "巳"),  // 立夏后
        ("2025-06-15", "午"),  // 芒种后
        ("2025-07-15", "未"),  // 小暑后
        ("2025-08-15", "申"),  // 立秋后
        ("2025-09-15", "酉"),  // 白露后
        ("2025-10-15", "戌"),  // 寒露后
        ("2025-11-15", "亥"),  // 立冬后
        ("2025-12-15", "子"),  // 大雪后
        ("2026-01-15", "丑"),  // 小寒后
    ];
    
    for (date_str, expected_zhi) in test_dates {
        let dt = chrono::NaiveDate::parse_from_str(date_str, "%Y-%m-%d")
            .unwrap()
            .and_hms_opt(12, 0, 0)
            .unwrap();
        let dt_utc = Utc.from_utc_datetime(&dt);
        
        let pillars = calc_bazi_pillars(&dt_utc, 116.4);
        let actual_zhi = DIZHI[pillars.month.1];
        
        println!("{}: 月支 {} (期望 {}), 黄经 {:.2}°",
            date_str, actual_zhi, expected_zhi, pillars.solar_longitude);
        
        assert_eq!(actual_zhi, expected_zhi,
            "{} 月支应为 {}, 实际为 {}", date_str, expected_zhi, actual_zhi);
    }
}
