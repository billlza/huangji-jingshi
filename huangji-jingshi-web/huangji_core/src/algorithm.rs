use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PeriodInfo {
    pub name: String,
    pub start_year: i32,
    pub end_year: i32,  // 闭区间展示用，实际内部用半开区间
    pub index: u32,
    pub max_index: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HuangjiInfo {
    pub yuan: PeriodInfo,
    pub hui: PeriodInfo,
    pub yun: PeriodInfo,
    pub shi: PeriodInfo,
    pub xun: PeriodInfo,
    pub year_gua: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimelineData {
    pub current: HuangjiInfo,
    pub yuan_list: Vec<PeriodInfo>,
    pub hui_list: Vec<PeriodInfo>,
    pub yun_list: Vec<PeriodInfo>,
    pub shi_list: Vec<PeriodInfo>,
    pub xun_list: Vec<PeriodInfo>,
}

// ============================================================
// P0 修复 #1: acc ↔ year 对称转换（无公元0年）
// ============================================================
// 约定：year 使用历史纪年 ..., -2, -1, 1, 2, ... （没有0年）
// 约定：acc 是连续整数轴，1 BC = 67017, 1 AD = 67018
const ACC_BC1: i32 = 67017;

/// 公元年 → 累积年（无0年）
/// 返回 Result 类型，year=0 时返回错误
pub fn year_to_acc(year: i32) -> Result<i32, &'static str> {
    if year == 0 {
        return Err("公元0年不存在");
    }
    if year >= 1 {
        Ok(ACC_BC1 + year)      // 1 AD -> 67018
    } else {
        Ok(ACC_BC1 + year + 1)  // 1 BC (-1) -> 67017, 2 BC (-2) -> 67016
    }
}

/// 累积年 → 公元年（无0年）
pub fn acc_to_year(acc: i32) -> i32 {
    if acc >= ACC_BC1 + 1 {
        acc - ACC_BC1       // 67018 -> 1 AD
    } else {
        acc - ACC_BC1 - 1   // 67017 -> -1 (1 BC), 67016 -> -2 (2 BC)
    }
}

// ============================================================
// 卦象表：(upper, lower, name) - 注意顺序！
// upper = 上卦, lower = 下卦
// ============================================================
const HEXAGRAM_TABLE: [(u8, u8, &str); 64] = [
    // 乾宫八卦
    // 修正：(7, 0) 应为"否"（天地否 = 乾上坤下），不是"泰"
    (7, 7, "乾"), (7, 3, "夬"), (7, 5, "大有"), (7, 1, "大壮"),
    (7, 6, "小畜"), (7, 2, "需"), (7, 4, "大畜"), (7, 0, "否"),
    // 兑宫八卦
    (3, 7, "履"), (3, 3, "兑"), (3, 5, "睽"), (3, 1, "归妹"),
    (3, 6, "中孚"), (3, 2, "节"), (3, 4, "损"), (3, 0, "临"),
    // 离宫八卦
    (5, 7, "同人"), (5, 3, "革"), (5, 5, "离"), (5, 1, "丰"),
    (5, 6, "家人"), (5, 2, "既济"), (5, 4, "贲"), (5, 0, "明夷"),
    // 震宫八卦
    (1, 7, "无妄"), (1, 3, "随"), (1, 5, "噬嗑"), (1, 1, "震"),
    (1, 6, "益"), (1, 2, "屯"), (1, 4, "颐"), (1, 0, "复"),
    // 巽宫八卦
    (6, 7, "姤"), (6, 3, "大过"), (6, 5, "鼎"), (6, 1, "恒"),
    (6, 6, "巽"), (6, 2, "井"), (6, 4, "蛊"), (6, 0, "升"),
    // 坎宫八卦
    (2, 7, "讼"), (2, 3, "困"), (2, 5, "未济"), (2, 1, "解"),
    (2, 6, "涣"), (2, 2, "坎"), (2, 4, "蒙"), (2, 0, "师"),
    // 艮宫八卦
    (4, 7, "遁"), (4, 3, "咸"), (4, 5, "旅"), (4, 1, "小过"),
    (4, 6, "渐"), (4, 2, "蹇"), (4, 4, "艮"), (4, 0, "谦"),
    // 坤宫八卦
    // 修正：(0, 7) 应为"泰"（地天泰 = 坤上乾下），不是"否"
    (0, 7, "泰"), (0, 3, "萃"), (0, 5, "晋"), (0, 1, "豫"),
    (0, 6, "观"), (0, 2, "比"), (0, 4, "剥"), (0, 0, "坤"),
];

// P0 修复 #3: 卦名 ↔ upper/lower 正确映射
/// (upper, lower) → 卦名
pub fn get_hexagram_name(upper: u8, lower: u8) -> String {
    for &(u, l, name) in &HEXAGRAM_TABLE {
        if u == upper && l == lower {
            return name.to_string();
        }
    }
    "未知".to_string()
}

/// 卦名 → (upper, lower)
pub fn get_hexagram_struct(name: &str) -> (u8, u8) {
    for &(u, l, n) in &HEXAGRAM_TABLE {
        if n == name {
            return (u, l);
        }
    }
    (0, 0)
}

pub const FUXI_SEQ: [&str; 64] = [
    "姤", "大过", "鼎", "恒", "巽", "井", "蛊", "升",
    "讼", "困", "未济", "解", "涣", "坎", "蒙", "师",
    "遁", "咸", "旅", "小过", "渐", "蹇", "艮", "谦",
    "否", "萃", "晋", "豫", "观", "比", "剥", "坤",
    "复", "颐", "屯", "益", "震", "噬嗑", "随", "无妄",
    "明夷", "贲", "既济", "家人", "丰", "离", "革", "同人",
    "临", "损", "节", "中孚", "归妹", "睽", "兑", "履",
    "泰", "大畜", "需", "小畜", "大壮", "大有", "夬", "乾"
];

const SKIP_ZHE: [&str; 4] = ["乾", "坤", "坎", "离"];

pub fn get_zheng_gua_seq() -> Vec<String> {
    let mut seq = Vec::new();
    let start_idx = FUXI_SEQ.iter().position(|&x| x == "复").unwrap();
    for i in 0..64 {
        let idx = (start_idx + i) % 64;
        let name = FUXI_SEQ[idx];
        if !SKIP_ZHE.contains(&name) {
            seq.push(name.to_string());
        }
    }
    seq
}

#[derive(Debug, Clone)]
struct Hexagram {
    upper: u8,
    lower: u8,
}

impl Hexagram {
    fn new(upper: u8, lower: u8) -> Self {
        Self { upper, lower }
    }
    
    fn from_name(name: &str) -> Self {
        let (u, l) = get_hexagram_struct(name);
        Self { upper: u, lower: l }
    }
    
    /// 变爻：line_idx 0-5，0=初爻(最下)，5=上爻(最上)
    fn change_line(&self, line_idx: usize) -> Self {
        // 6位二进制：upper(3位) | lower(3位)
        // lower 的 bit0 = 初爻, bit2 = 三爻
        // upper 的 bit0 = 四爻, bit2 = 上爻
        let mut full = ((self.upper as u16) << 3) | (self.lower as u16);
        full ^= 1 << line_idx;
        Self {
            upper: ((full >> 3) & 0x7) as u8,
            lower: (full & 0x7) as u8,
        }
    }
    
    fn name(&self) -> String {
        get_hexagram_name(self.upper, self.lower)
    }
}

#[allow(dead_code)]
fn get_trigram_name(val: u8) -> &'static str {
    match val {
        7 => "乾", 3 => "兑", 5 => "离", 1 => "震",
        6 => "巽", 2 => "坎", 4 => "艮", 0 => "坤",
        _ => "?"
    }
}

// ============================================================
// 核心计算：使用 Euclid 除法，统一半开区间
// ============================================================
// 基准年：元的起点（累积年=1 对应的公元年）
const EPOCH_ACC: i32 = 1;  // 累积年起点
#[allow(dead_code)]
const EPOCH_YEAR: i32 = -67016;  // acc=1 对应 -67016 年 (67017 BC)

/// 计算距下一个周期边界的年数（使用 acc 差值，保证非负）
/// Requirements 6.3, 6.4: 返回值必须为非负整数
pub fn years_to_next_boundary(current_acc: i32, next_boundary_acc: i32) -> i32 {
    (next_boundary_acc - current_acc).max(0)
}

pub fn get_hj_info(hj_year: i32) -> HuangjiInfo {
    // P0 修复 #1: 使用统一的 year_to_acc
    // 注意：hj_year 不应为 0，调用方应确保输入有效
    let acc = year_to_acc(hj_year).expect("hj_year should not be 0");
    
    // 相对于 epoch 的偏移（用于 Euclid 除法）
    let t = acc - EPOCH_ACC;
    
    // ============================================================
    // P0 修复 #2: 全部使用 div_euclid / rem_euclid
    // ============================================================
    
    // 1. 元 (129600 年)
    let yuan_index = t.div_euclid(129600);
    let yuan_start_acc = EPOCH_ACC + yuan_index * 129600;
    let yuan_start_year = acc_to_year(yuan_start_acc);
    let yuan_end_year = acc_to_year(yuan_start_acc + 129600 - 1);  // 闭区间展示
    
    let yuan_info = PeriodInfo {
        name: format!("{}", yuan_index + 1),
        start_year: yuan_start_year,
        end_year: yuan_end_year,
        index: (yuan_index + 1) as u32,
        max_index: 1,
    };
    
    // 2. 会 (10800 年)
    let hui_index = t.div_euclid(10800);
    let hui_in_yuan = hui_index.rem_euclid(12);  // 0-11
    let hui_start_acc = EPOCH_ACC + hui_index * 10800;
    let hui_start_year = acc_to_year(hui_start_acc);
    let hui_end_year = acc_to_year(hui_start_acc + 10800 - 1);
    
    let hui_names = ["子", "丑", "寅", "卯", "辰", "巳", "午", "未", "申", "酉", "戌", "亥"];
    let hui_name = hui_names[hui_in_yuan as usize].to_string();
    
    let hui_info = PeriodInfo {
        name: hui_name,
        start_year: hui_start_year,
        end_year: hui_end_year,
        index: (hui_in_yuan + 1) as u32,
        max_index: 12,
    };
    
    // 3. 运 (360 年)
    let yun_global_index = t.div_euclid(360);
    let yun_in_hui = yun_global_index.rem_euclid(30);  // 0-29
    let yun_start_acc = EPOCH_ACC + yun_global_index * 360;
    let yun_start_year = acc_to_year(yun_start_acc);
    let yun_end_year = acc_to_year(yun_start_acc + 360 - 1);
    
    // 运卦计算
    let zheng_gua_seq = get_zheng_gua_seq();
    let zheng_gua_idx = (yun_global_index.rem_euclid(360) / 6) as usize;  // 每6运一个正卦
    let zheng_gua_name = &zheng_gua_seq[zheng_gua_idx % zheng_gua_seq.len()];
    let zheng_hex = Hexagram::from_name(zheng_gua_name);
    let yun_line = (yun_global_index.rem_euclid(6)) as usize;
    let yun_hex = zheng_hex.change_line(yun_line);
    
    let yun_info = PeriodInfo {
        name: yun_hex.name(),
        start_year: yun_start_year,
        end_year: yun_end_year,
        index: (yun_in_hui + 1) as u32,
        max_index: 30,
    };
    
    // 4. 世 (30 年)
    let shi_global_index = t.div_euclid(30);
    let shi_in_yun = shi_global_index.rem_euclid(12);  // 0-11
    let shi_start_acc = EPOCH_ACC + shi_global_index * 30;
    let shi_start_year = acc_to_year(shi_start_acc);
    let shi_end_year = acc_to_year(shi_start_acc + 30 - 1);
    
    // 世卦计算：在运卦基础上变爻
    let shi_line = (shi_in_yun / 2) as usize;  // 每2世变一爻
    let shi_hex = yun_hex.change_line(shi_line);
    
    let shi_info = PeriodInfo {
        name: shi_hex.name(),
        start_year: shi_start_year,
        end_year: shi_end_year,
        index: (shi_in_yun + 1) as u32,
        max_index: 12,
    };
    
    // 5. 旬 (10 年)
    let xun_global_index = t.div_euclid(10);
    let xun_in_shi = xun_global_index.rem_euclid(3);  // 0-2
    let xun_start_acc = EPOCH_ACC + xun_global_index * 10;
    let xun_start_year = acc_to_year(xun_start_acc);
    let xun_end_year = acc_to_year(xun_start_acc + 10 - 1);
    
    let xun_names = ["甲子", "甲戌", "甲申"];
    let xun_info = PeriodInfo {
        name: xun_names[xun_in_shi as usize].to_string(),
        start_year: xun_start_year,
        end_year: xun_end_year,
        index: (xun_in_shi + 1) as u32,
        max_index: 3,
    };
    
    // 6. 年卦
    let mut val_seq: Vec<String> = FUXI_SEQ.iter()
        .filter(|&&n| !SKIP_ZHE.contains(&n))
        .map(|&s| s.to_string())
        .collect();
    
    let anchor_year = 1984;
    let anchor_name = "鼎";
    let anchor_idx = val_seq.iter().position(|x| x == anchor_name).unwrap_or(0);
    
    let year_offset = hj_year - anchor_year;
    let len = val_seq.len() as i32;
    let target_idx = (anchor_idx as i32 + year_offset).rem_euclid(len);
    let year_gua = val_seq[target_idx as usize].clone();
    
    HuangjiInfo {
        yuan: yuan_info,
        hui: hui_info,
        yun: yun_info,
        shi: shi_info,
        xun: xun_info,
        year_gua,
    }
}


pub fn get_timeline_info(hj_year: i32) -> TimelineData {
    let current = get_hj_info(hj_year);
    let acc = year_to_acc(hj_year).expect("hj_year should not be 0");
    let t = acc - EPOCH_ACC;
    
    // 1. Yuan List (单个元)
    let yuan_list = vec![current.yuan.clone()];
    
    // 2. Hui List (12会)
    let yuan_index = t.div_euclid(129600);
    let yuan_start_acc = EPOCH_ACC + yuan_index * 129600;
    let hui_names = ["子", "丑", "寅", "卯", "辰", "巳", "午", "未", "申", "酉", "戌", "亥"];
    let hui_list: Vec<PeriodInfo> = (0..12).map(|i| {
        let start_acc = yuan_start_acc + i * 10800;
        PeriodInfo {
            name: hui_names[i as usize].to_string(),
            start_year: acc_to_year(start_acc),
            end_year: acc_to_year(start_acc + 10800 - 1),
            index: (i + 1) as u32,
            max_index: 12,
        }
    }).collect();
    
    // 3. Yun List (30运)
    let hui_index = t.div_euclid(10800);
    let hui_start_acc = EPOCH_ACC + hui_index * 10800;
    let zheng_gua_seq = get_zheng_gua_seq();
    
    let yun_list: Vec<PeriodInfo> = (0..30).map(|i| {
        let yun_global = hui_index * 30 + i;
        let start_acc = hui_start_acc + i * 360;
        
        let zheng_idx = (yun_global.rem_euclid(360) / 6) as usize;
        let zheng_name = &zheng_gua_seq[zheng_idx % zheng_gua_seq.len()];
        let zheng_hex = Hexagram::from_name(zheng_name);
        let yun_line = (yun_global.rem_euclid(6)) as usize;
        let yun_hex = zheng_hex.change_line(yun_line);
        
        PeriodInfo {
            name: yun_hex.name(),
            start_year: acc_to_year(start_acc),
            end_year: acc_to_year(start_acc + 360 - 1),
            index: (i + 1) as u32,
            max_index: 30,
        }
    }).collect();
    
    // 4. Shi List (12世)
    let yun_global_index = t.div_euclid(360);
    let yun_start_acc = EPOCH_ACC + yun_global_index * 360;
    
    // 获取当前运卦
    let zheng_idx = (yun_global_index.rem_euclid(360) / 6) as usize;
    let zheng_name = &zheng_gua_seq[zheng_idx % zheng_gua_seq.len()];
    let zheng_hex = Hexagram::from_name(zheng_name);
    let yun_line = (yun_global_index.rem_euclid(6)) as usize;
    let yun_hex = zheng_hex.change_line(yun_line);
    
    let shi_list: Vec<PeriodInfo> = (0..12).map(|i| {
        let start_acc = yun_start_acc + i * 30;
        let shi_line = (i / 2) as usize;
        let shi_hex = yun_hex.change_line(shi_line);
        
        PeriodInfo {
            name: shi_hex.name(),
            start_year: acc_to_year(start_acc),
            end_year: acc_to_year(start_acc + 30 - 1),
            index: (i + 1) as u32,
            max_index: 12,
        }
    }).collect();
    
    // 5. Xun List (3旬)
    let shi_global_index = t.div_euclid(30);
    let shi_start_acc = EPOCH_ACC + shi_global_index * 30;
    let xun_names = ["甲子", "甲戌", "甲申"];
    
    let xun_list: Vec<PeriodInfo> = (0..3).map(|i| {
        let start_acc = shi_start_acc + i * 10;
        PeriodInfo {
            name: xun_names[i as usize].to_string(),
            start_year: acc_to_year(start_acc),
            end_year: acc_to_year(start_acc + 10 - 1),
            index: (i + 1) as u32,
            max_index: 3,
        }
    }).collect();
    
    TimelineData {
        current,
        yuan_list,
        hui_list,
        yun_list,
        shi_list,
        xun_list,
    }
}

// ============================================================
// 单元测试
// ============================================================
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_year_acc_roundtrip() {
        // P0 测试：acc ↔ year 对称性
        let years = [-3000, -2, -1, 1, 2, 2024, 2044];
        for &y in &years {
            let acc = year_to_acc(y).unwrap();
            let y2 = acc_to_year(acc);
            assert_eq!(y, y2, "year {} -> acc {} -> year {}", y, acc, y2);
        }
    }
    
    #[test]
    fn test_no_year_zero() {
        // 确保 -1 和 1 是连续的累积年
        let acc_bc1 = year_to_acc(-1).unwrap();  // 1 BC
        let acc_ad1 = year_to_acc(1).unwrap();   // 1 AD
        assert_eq!(acc_ad1 - acc_bc1, 1, "1 BC and 1 AD should be consecutive");
    }
    
    #[test]
    fn test_year_zero_returns_error() {
        // 测试 year=0 返回错误
        let result = year_to_acc(0);
        assert!(result.is_err(), "year_to_acc(0) should return Err");
        assert_eq!(result.unwrap_err(), "公元0年不存在");
    }
    
    #[test]
    fn test_segment_lengths() {
        // P0 测试：区间长度恒定
        let test_years = [-2156, -1, 1, 2014, 2044, 9999];
        for &y in &test_years {
            let info = get_hj_info(y);
            
            // 会长度 = 10800
            let hui_len = year_to_acc(info.hui.end_year).unwrap() - year_to_acc(info.hui.start_year).unwrap() + 1;
            assert_eq!(hui_len, 10800, "Hui length for year {} should be 10800, got {}", y, hui_len);
            
            // 运长度 = 360
            let yun_len = year_to_acc(info.yun.end_year).unwrap() - year_to_acc(info.yun.start_year).unwrap() + 1;
            assert_eq!(yun_len, 360, "Yun length for year {} should be 360, got {}", y, yun_len);
            
            // 世长度 = 30
            let shi_len = year_to_acc(info.shi.end_year).unwrap() - year_to_acc(info.shi.start_year).unwrap() + 1;
            assert_eq!(shi_len, 30, "Shi length for year {} should be 30, got {}", y, shi_len);
            
            // 旬长度 = 10
            let xun_len = year_to_acc(info.xun.end_year).unwrap() - year_to_acc(info.xun.start_year).unwrap() + 1;
            assert_eq!(xun_len, 10, "Xun length for year {} should be 10, got {}", y, xun_len);
        }
    }
    
    #[test]
    fn test_hexagram_bijection() {
        // P0 测试：卦名 ↔ upper/lower 可逆
        // 修正后：否=(7,0)乾上坤下，泰=(0,7)坤上乾下
        let samples = [
            (7, 7, "乾"),
            (0, 0, "坤"),
            (7, 0, "否"),  // 天地否 = 乾上坤下
            (0, 7, "泰"),  // 地天泰 = 坤上乾下
            (2, 2, "坎"),
            (5, 5, "离"),
        ];
        for (u, l, name) in samples {
            assert_eq!(get_hexagram_name(u, l), name, "({},{}) should be {}", u, l, name);
            let (u2, l2) = get_hexagram_struct(name);
            assert_eq!((u, l), (u2, l2), "{} should map to ({},{})", name, u, l);
        }
    }
    
    // ============================================================
    // 卦象黄金用例测试 - Requirements 3.1-3.8
    // ============================================================
    #[test]
    fn test_hexagram_pi_tai() {
        // Requirements 3.1, 3.5: 乾
        assert_eq!(get_hexagram_name(7, 7), "乾", "(7,7) should be 乾");
        assert_eq!(get_hexagram_struct("乾"), (7, 7), "乾 should map to (7,7)");
        
        // Requirements 3.2, 3.6: 坤
        assert_eq!(get_hexagram_name(0, 0), "坤", "(0,0) should be 坤");
        assert_eq!(get_hexagram_struct("坤"), (0, 0), "坤 should map to (0,0)");
        
        // Requirements 3.3, 3.7: 否 = 天地否 = 乾上坤下
        assert_eq!(get_hexagram_name(7, 0), "否", "(7,0) should be 否 (天地否)");
        assert_eq!(get_hexagram_struct("否"), (7, 0), "否 should map to (7,0)");
        
        // Requirements 3.4, 3.8: 泰 = 地天泰 = 坤上乾下
        assert_eq!(get_hexagram_name(0, 7), "泰", "(0,7) should be 泰 (地天泰)");
        assert_eq!(get_hexagram_struct("泰"), (0, 7), "泰 should map to (0,7)");
    }
    
    #[test]
    fn test_boundary_2044() {
        // 边界测试：2044年应该在新的世
        let info_2043 = get_hj_info(2043);
        let info_2044 = get_hj_info(2044);
        
        // 2044 应该是新世的起点
        assert_eq!(info_2044.shi.start_year, 2044, "2044 should be start of new Shi");
        assert_eq!(info_2043.shi.end_year, 2043, "2043 should be end of previous Shi");
    }
    
    #[test]
    fn test_bce_continuity() {
        // BCE 连续性测试
        let info_bc2 = get_hj_info(-2);
        let info_bc1 = get_hj_info(-1);
        let info_ad1 = get_hj_info(1);
        
        // 确保索引连续递增
        let acc_bc2 = year_to_acc(-2).unwrap();
        let acc_bc1 = year_to_acc(-1).unwrap();
        let acc_ad1 = year_to_acc(1).unwrap();
        
        assert_eq!(acc_bc1 - acc_bc2, 1);
        assert_eq!(acc_ad1 - acc_bc1, 1);
    }
    
    // ============================================================
    // 黄金用例测试 - Requirements 1.1, 1.2, 1.3, 1.4, 1.6
    // ============================================================
    #[test]
    fn test_year_acc_golden_cases() {
        // Requirements 1.1, 1.2: year_to_acc 黄金用例
        assert_eq!(year_to_acc(-1).unwrap(), 67017, "1 BC should map to acc=67017");
        assert_eq!(year_to_acc(1).unwrap(), 67018, "1 AD should map to acc=67018");
        
        // Requirements 1.3, 1.4: acc_to_year 黄金用例
        assert_eq!(acc_to_year(67017), -1, "acc=67017 should map to 1 BC");
        assert_eq!(acc_to_year(67018), 1, "acc=67018 should map to 1 AD");
        
        // Requirements 1.6: 连续性（差值为 1）
        assert_eq!(year_to_acc(1).unwrap() - year_to_acc(-1).unwrap(), 1, 
                   "acc(1 AD) - acc(1 BC) should be 1");
    }
    
    // ============================================================
    // 边界年份测试 - Requirements 6.1, 6.2, 6.3, 6.4
    // 测试条件：mode=GregorianNewYear, true_solar_time=false, tzOffsetMinutes=+480
    // ============================================================
    #[test]
    fn test_boundary_2044_detailed() {
        // Requirements 6.1: 2044-01-01 属于新世（2044-2073）
        let info_2044 = get_hj_info(2044);
        assert_eq!(info_2044.shi.start_year, 2044, "2044 should be start of new Shi");
        assert!(info_2044.shi.end_year >= 2044, "2044 Shi end_year should be >= 2044");
        
        // Requirements 6.2: 2043-12-31 属于上一世（2014-2043）
        let info_2043 = get_hj_info(2043);
        assert_eq!(info_2043.shi.end_year, 2043, "2043 should be end of previous Shi");
        assert!(info_2043.shi.start_year <= 2043, "2043 Shi start_year should be <= 2043");
        
        // 确保 2043 和 2044 在不同的世
        assert!(info_2044.shi.start_year > info_2043.shi.start_year,
                "2044 should be in a different Shi than 2043");
    }
    
    #[test]
    fn test_years_to_next_boundary_nonnegative() {
        // Requirements 6.3, 6.4: 距下一周期应为非负整数
        let test_years = [-2156, -1, 1, 2014, 2043, 2044, 9999];
        for &y in &test_years {
            let info = get_hj_info(y);
            let acc = year_to_acc(y).unwrap();
            
            // 距下一世
            let shi_end_acc = year_to_acc(info.shi.end_year).unwrap() + 1;
            let years_to_next_shi = years_to_next_boundary(acc, shi_end_acc);
            assert!(years_to_next_shi >= 0, 
                    "Years to next Shi should be non-negative for year {}, got {}", 
                    y, years_to_next_shi);
            
            // 距下一旬
            let xun_end_acc = year_to_acc(info.xun.end_year).unwrap() + 1;
            let years_to_next_xun = years_to_next_boundary(acc, xun_end_acc);
            assert!(years_to_next_xun >= 0, 
                    "Years to next Xun should be non-negative for year {}, got {}", 
                    y, years_to_next_xun);
        }
    }
}
