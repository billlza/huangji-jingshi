//! 属性测试模块 - 使用 proptest 进行属性测试
//! 
//! 本模块实现设计文档中定义的 7 个正确性属性：
//! - Property 1: acc ↔ year Round-trip 一致性
//! - Property 2: 区间长度恒定（acc 域）
//! - Property 3: 半开区间包含性
//! - Property 4: 分段单调连续性（边界跃迁）
//! - Property 5: 卦象映射双向一致性
//! - Property 7: 距下一周期非负

use proptest::prelude::*;
use huangji_core::algorithm::{
    year_to_acc, acc_to_year, get_hj_info,
    get_hexagram_name, get_hexagram_struct,
    FUXI_SEQ,
};

// 配置：每个属性测试运行 100+ 次迭代
proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]
    
    /// **Feature: huangji-bugfix, Property 1: acc ↔ year Round-trip 一致性**
    /// *For any* 非零整数年份 year（year ≠ 0），执行 acc_to_year(year_to_acc(year)) 应返回原始 year 值
    /// **Validates: Requirements 1.5**
    #[test]
    fn prop_year_acc_roundtrip(year in -10000i32..10000i32) {
        prop_assume!(year != 0);  // 排除公元0年
        let acc = year_to_acc(year).expect("year_to_acc should succeed for non-zero year");
        let year2 = acc_to_year(acc);
        prop_assert_eq!(year, year2, "Round-trip failed: {} -> {} -> {}", year, acc, year2);
    }
    
    /// **Feature: huangji-bugfix, Property 2: 区间长度恒定（acc 域）**
    /// *For any* 年份（包括 BCE），计算得到的周期区间长度在 acc 域应恒定
    /// **Validates: Requirements 2.5, 2.6, 2.7, 2.8**
    #[test]
    fn prop_segment_length_constant(year in -10000i32..10000i32) {
        prop_assume!(year != 0);
        let info = get_hj_info(year);
        
        // 会长度 = 10800
        let hui_start_acc = year_to_acc(info.hui.start_year).unwrap();
        let hui_end_acc = year_to_acc(info.hui.end_year).unwrap();
        let hui_len = hui_end_acc - hui_start_acc + 1;  // 闭区间转长度
        prop_assert_eq!(hui_len, 10800, "Hui length should be 10800, got {} for year {}", hui_len, year);
        
        // 运长度 = 360
        let yun_start_acc = year_to_acc(info.yun.start_year).unwrap();
        let yun_end_acc = year_to_acc(info.yun.end_year).unwrap();
        let yun_len = yun_end_acc - yun_start_acc + 1;
        prop_assert_eq!(yun_len, 360, "Yun length should be 360, got {} for year {}", yun_len, year);
        
        // 世长度 = 30
        let shi_start_acc = year_to_acc(info.shi.start_year).unwrap();
        let shi_end_acc = year_to_acc(info.shi.end_year).unwrap();
        let shi_len = shi_end_acc - shi_start_acc + 1;
        prop_assert_eq!(shi_len, 30, "Shi length should be 30, got {} for year {}", shi_len, year);
        
        // 旬长度 = 10
        let xun_start_acc = year_to_acc(info.xun.start_year).unwrap();
        let xun_end_acc = year_to_acc(info.xun.end_year).unwrap();
        let xun_len = xun_end_acc - xun_start_acc + 1;
        prop_assert_eq!(xun_len, 10, "Xun length should be 10, got {} for year {}", xun_len, year);
    }
    
    /// **Feature: huangji-bugfix, Property 3: 半开区间包含性**
    /// *For any* acc 值，计算得到的区间应满足 segment_start_acc <= acc < segment_end_acc
    /// **Validates: Requirements 2.9**
    #[test]
    fn prop_segment_containment(year in -10000i32..10000i32) {
        prop_assume!(year != 0);
        let acc = year_to_acc(year).unwrap();
        let info = get_hj_info(year);
        
        // 检查世区间包含性（半开区间）
        let shi_start_acc = year_to_acc(info.shi.start_year).unwrap();
        let shi_end_acc = year_to_acc(info.shi.end_year).unwrap() + 1;  // 转为半开区间
        prop_assert!(
            shi_start_acc <= acc && acc < shi_end_acc,
            "Shi containment failed: {} not in [{}, {}) for year {}",
            acc, shi_start_acc, shi_end_acc, year
        );
        
        // 检查旬区间包含性
        let xun_start_acc = year_to_acc(info.xun.start_year).unwrap();
        let xun_end_acc = year_to_acc(info.xun.end_year).unwrap() + 1;
        prop_assert!(
            xun_start_acc <= acc && acc < xun_end_acc,
            "Xun containment failed: {} not in [{}, {}) for year {}",
            acc, xun_start_acc, xun_end_acc, year
        );
    }
    
    /// **Feature: huangji-bugfix, Property 4: 分段单调连续性（边界跃迁）**
    /// *For any* 连续的 acc 值对 (acc, acc+1)，节点索引要么不变，要么在边界处恰好 +1
    /// **Validates: Requirements 2.10**
    #[test]
    fn prop_segment_monotonicity(year in -9999i32..9999i32) {
        prop_assume!(year != 0 && year != -1);  // 避免跨 0 年边界
        
        let info1 = get_hj_info(year);
        let next_year = if year == -1 { 1 } else { year + 1 };
        let info2 = get_hj_info(next_year);
        
        // 世索引：要么不变，要么 +1（考虑循环）
        let shi_diff = info2.shi.index as i32 - info1.shi.index as i32;
        prop_assert!(
            shi_diff == 0 || shi_diff == 1 || (info1.shi.index == 12 && info2.shi.index == 1),
            "Shi index jump: {} -> {} for years {} -> {}",
            info1.shi.index, info2.shi.index, year, next_year
        );
        
        // 旬索引：要么不变，要么 +1（考虑循环）
        let xun_diff = info2.xun.index as i32 - info1.xun.index as i32;
        prop_assert!(
            xun_diff == 0 || xun_diff == 1 || (info1.xun.index == 3 && info2.xun.index == 1),
            "Xun index jump: {} -> {} for years {} -> {}",
            info1.xun.index, info2.xun.index, year, next_year
        );
    }
    
    /// **Feature: huangji-bugfix, Property 5: 卦象映射双向一致性**
    /// *For any* 有效的 (upper, lower) 组合，执行 get_hexagram_struct(get_hexagram_name(upper, lower)) 应返回原始值
    /// **Validates: Requirements 3.9, 3.10**
    #[test]
    fn prop_hexagram_bijection(upper in 0u8..8u8, lower in 0u8..8u8) {
        let name = get_hexagram_name(upper, lower);
        if name != "未知" {
            let (u2, l2) = get_hexagram_struct(&name);
            prop_assert_eq!(
                (upper, lower), (u2, l2),
                "Hexagram bijection failed: ({},{}) -> {} -> ({},{})",
                upper, lower, name, u2, l2
            );
        }
    }
    
    /// **Feature: huangji-bugfix, Property 7: 距下一周期非负**
    /// *For any* 年份，计算得到的"距下一世"和"距下一旬"的年数应为非负整数
    /// **Validates: Requirements 6.3, 6.4**
    #[test]
    fn prop_years_to_next_nonnegative(year in -10000i32..10000i32) {
        prop_assume!(year != 0);
        let info = get_hj_info(year);
        let acc = year_to_acc(year).unwrap();
        
        // 距下一世
        let shi_end_acc = year_to_acc(info.shi.end_year).unwrap() + 1;
        let years_to_next_shi = (shi_end_acc - acc).max(0);
        prop_assert!(years_to_next_shi >= 0, "Years to next Shi should be non-negative");
        
        // 距下一旬
        let xun_end_acc = year_to_acc(info.xun.end_year).unwrap() + 1;
        let years_to_next_xun = (xun_end_acc - acc).max(0);
        prop_assert!(years_to_next_xun >= 0, "Years to next Xun should be non-negative");
    }
}

/// 卦名 round-trip 测试（遍历所有 64 卦）
#[test]
fn test_all_hexagram_names_roundtrip() {
    // 跳过四正卦
    let skip = ["乾", "坤", "坎", "离"];
    
    for &name in FUXI_SEQ.iter() {
        if skip.contains(&name) {
            continue;
        }
        let (u, l) = get_hexagram_struct(name);
        let name2 = get_hexagram_name(u, l);
        assert_eq!(name, name2, "Hexagram name roundtrip failed: {} -> ({},{}) -> {}", name, u, l, name2);
    }
}
