use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PeriodInfo {
    pub name: String,
    pub start_year: i32,
    pub end_year: i32,
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

#[allow(dead_code)]
#[derive(Debug, Clone)]
struct Hexagram {
    name: String,
    upper: u8,
    lower: u8,
}

impl Hexagram {
    fn new(name: &str, upper: u8, lower: u8) -> Self {
        Self { name: name.to_string(), upper, lower }
    }
    
    fn change_line(&self, line_idx_0_based: usize) -> (u8, u8) {
        let mut full = (self.upper << 3) | self.lower;
        full ^= 1 << line_idx_0_based;
        ((full >> 3) & 0x7, full & 0x7)
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

fn get_hexagram_name(upper: u8, lower: u8) -> String {
    let combinations = [
        (7,7,"乾"), (7,3,"夬"), (7,5,"大有"), (7,1,"大壮"), (7,6,"小畜"), (7,2,"需"), (7,4,"大畜"), (7,0,"泰"),
        (3,7,"履"), (3,3,"兑"), (3,5,"睽"), (3,1,"归妹"), (3,6,"中孚"), (3,2,"节"), (3,4,"损"), (3,0,"临"),
        (5,7,"同人"), (5,3,"革"), (5,5,"离"), (5,1,"丰"), (5,6,"家人"), (5,2,"既济"), (5,4,"贲"), (5,0,"明夷"),
        (1,7,"无妄"), (1,3,"随"), (1,5,"噬嗑"), (1,1,"震"), (1,6,"益"), (1,2,"屯"), (1,4,"颐"), (1,0,"复"),
        (6,7,"姤"), (6,3,"大过"), (6,5,"鼎"), (6,1,"恒"), (6,6,"巽"), (6,2,"井"), (6,4,"蛊"), (6,0,"升"),
        (2,7,"讼"), (2,3,"困"), (2,5,"未济"), (2,1,"解"), (2,6,"涣"), (2,2,"坎"), (2,4,"蒙"), (2,0,"师"),
        (4,7,"遁"), (4,3,"咸"), (4,5,"旅"), (4,1,"小过"), (4,6,"渐"), (4,2,"蹇"), (4,4,"艮"), (4,0,"谦"),
        (0,7,"否"), (0,3,"萃"), (0,5,"晋"), (0,1,"豫"), (0,6,"观"), (0,2,"比"), (0,4,"剥"), (0,0,"坤"),
    ];
    
    for (l, u, n) in combinations.iter() {
        if *u == upper && *l == lower {
            return n.to_string();
        }
    }
    "未知".to_string()
}

pub fn get_hexagram_struct(name: &str) -> (u8, u8) {
     let combinations = [
        (7,7,"乾"), (7,3,"夬"), (7,5,"大有"), (7,1,"大壮"), (7,6,"小畜"), (7,2,"需"), (7,4,"大畜"), (7,0,"泰"),
        (3,7,"履"), (3,3,"兑"), (3,5,"睽"), (3,1,"归妹"), (3,6,"中孚"), (3,2,"节"), (3,4,"损"), (3,0,"临"),
        (5,7,"同人"), (5,3,"革"), (5,5,"离"), (5,1,"丰"), (5,6,"家人"), (5,2,"既济"), (5,4,"贲"), (5,0,"明夷"),
        (1,7,"无妄"), (1,3,"随"), (1,5,"噬嗑"), (1,1,"震"), (1,6,"益"), (1,2,"屯"), (1,4,"颐"), (1,0,"复"),
        (6,7,"姤"), (6,3,"大过"), (6,5,"鼎"), (6,1,"恒"), (6,6,"巽"), (6,2,"井"), (6,4,"蛊"), (6,0,"升"),
        (2,7,"讼"), (2,3,"困"), (2,5,"未济"), (2,1,"解"), (2,6,"涣"), (2,2,"坎"), (2,4,"蒙"), (2,0,"师"),
        (4,7,"遁"), (4,3,"咸"), (4,5,"旅"), (4,1,"小过"), (4,6,"渐"), (4,2,"蹇"), (4,4,"艮"), (4,0,"谦"),
        (0,7,"否"), (0,3,"萃"), (0,5,"晋"), (0,1,"豫"), (0,6,"观"), (0,2,"比"), (0,4,"剥"), (0,0,"坤"),
    ];
    for (l, u, n) in combinations.iter() {
        if *n == name {
            return (*u, *l);
        }
    }
    (0, 0)
}

pub fn get_hj_info(year_ad: i32) -> HuangjiInfo {
    // 1. Accumulated Years
    // 1 BC = 67017. 1 AD = 67018.
    let acc = if year_ad > 0 { 67017 + year_ad } else { 67017 + year_ad + 1 };
    
    // 2. Yuan (129600)
    let yuan_info = PeriodInfo {
        name: "1".to_string(),
        start_year: -67016,
        end_year: -67016 + 129600 - 1,
        index: 1,
        max_index: 1,
    };
    
    // 3. Hui (10800)
    let hui_idx = (acc - 1) / 10800 + 1;
    let hui_names = ["子", "丑", "寅", "卯", "辰", "巳", "午", "未", "申", "酉", "戌", "亥"];
    let hui_name_char = if (1..=12).contains(&hui_idx) { hui_names[(hui_idx - 1) as usize] } else { "?" };
    let hui_name = hui_name_char.to_string();
    
    let hui_start_acc = (hui_idx - 1) * 10800 + 1;
    let hui_start_ad = hui_start_acc - 67017;
    let hui_end_ad = hui_start_ad + 10799;
    
    let hui_info = PeriodInfo {
        name: hui_name,
        start_year: hui_start_ad,
        end_year: hui_end_ad,
        index: hui_idx as u32,
        max_index: 12,
    };
    
    // 4. Yun (360)
    let hui_offset = (acc - 1) % 10800;
    let yun_idx = hui_offset / 360 + 1;
    
    let zheng_gua_seq = get_zheng_gua_seq();
    let global_yun_idx = (hui_idx - 1) * 30 + (yun_idx - 1);
    let zheng_gua_idx = global_yun_idx / 6;
    let zheng_gua_name = &zheng_gua_seq[zheng_gua_idx as usize];
    
    let (zu, zl) = get_hexagram_struct(zheng_gua_name);
    let (yu, yl) = Hexagram::new(zheng_gua_name, zu, zl).change_line((global_yun_idx % 6) as usize);
    let yun_gua_name = get_hexagram_name(yu, yl);
    
    let yun_start_acc = (global_yun_idx * 360) + 1; 
    let yun_start_ad = yun_start_acc - 67017;
    let yun_end_ad = yun_start_ad + 359;
    
    let yun_info = PeriodInfo {
        name: yun_gua_name,
        start_year: yun_start_ad,
        end_year: yun_end_ad,
        index: yun_idx as u32,
        max_index: 30,
    };
    
    // 5. Shi (30)
    let yun_offset = hui_offset % 360;
    let shi_idx = yun_offset / 30 + 1;
    
    let (su, sl) = Hexagram::new(&yun_info.name, yu, yl).change_line(((shi_idx - 1) / 2) as usize);
    let shi_gua_name = get_hexagram_name(su, sl);
    
    let shi_start_acc = yun_start_acc + (shi_idx - 1) * 30;
    let shi_start_ad = shi_start_acc - 67017;
    let shi_end_ad = shi_start_ad + 29;
    
    let shi_info = PeriodInfo {
        name: shi_gua_name,
        start_year: shi_start_ad,
        end_year: shi_end_ad,
        index: shi_idx as u32,
        max_index: 12,
    };

    // 6. Xun (10)
    let shi_offset = yun_offset % 30;
    let xun_idx = shi_offset / 10 + 1;
    let xun_start_ad = shi_start_ad + (xun_idx - 1) * 10;
    let xun_end_ad = xun_start_ad + 9;
    
    let xun_info = PeriodInfo {
        name: format!("第{}旬", xun_idx),
        start_year: xun_start_ad,
        end_year: xun_end_ad,
        index: xun_idx as u32,
        max_index: 3,
    };
    
    // 7. Value Year Hexagram
    let mut val_seq = Vec::new();
    for name in FUXI_SEQ.iter() {
        if !SKIP_ZHE.contains(name) {
            val_seq.push(name.to_string());
        }
    }
    
    let anchor_year = 1984;
    let anchor_name = "鼎";
    let anchor_idx = val_seq.iter().position(|x| x == anchor_name).unwrap_or(0);
    
    let year_offset = year_ad - anchor_year;
    let len = val_seq.len() as i32;
    let mut target_idx = (anchor_idx as i32 + year_offset) % len;
    if target_idx < 0 {
        target_idx += len;
    }
    
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

pub fn get_timeline_info(year_ad: i32) -> TimelineData {
    let current = get_hj_info(year_ad);
    
    // 1. Yuan List (Single)
    let yuan_list = vec![current.yuan.clone()];
    
    // 2. Hui List (12)
    // Based on Yuan start year.
    let yuan_start_ad = current.yuan.start_year;
    let mut hui_list = Vec::new();
    let hui_names = ["子", "丑", "寅", "卯", "辰", "巳", "午", "未", "申", "酉", "戌", "亥"];
    for i in 1..=12 {
        let start = yuan_start_ad + (i - 1) * 10800;
        let end = start + 10799;
        hui_list.push(PeriodInfo {
            name: hui_names[(i - 1) as usize].to_string(),
            start_year: start,
            end_year: end,
            index: i as u32,
            max_index: 12,
        });
    }
    
    // 3. Yun List (30)
    // Based on current Hui start year.
    let hui_start_ad = current.hui.start_year;
    let hui_idx = current.hui.index;
    let mut yun_list = Vec::new();
    let zheng_gua_seq = get_zheng_gua_seq();
    
    for i in 1..=30 {
        let start = hui_start_ad + (i - 1) * 360;
        let end = start + 359;
        
        let global_yun_idx = (hui_idx - 1) * 30 + (i as u32 - 1);
        let zheng_gua_idx = global_yun_idx / 6;
        let zheng_gua_name = &zheng_gua_seq[zheng_gua_idx as usize];
        
        let (zu, zl) = get_hexagram_struct(zheng_gua_name);
        let (yu, yl) = Hexagram::new(zheng_gua_name, zu, zl).change_line((global_yun_idx % 6) as usize);
        let yun_gua_name = get_hexagram_name(yu, yl);
        
        yun_list.push(PeriodInfo {
            name: yun_gua_name,
            start_year: start,
            end_year: end,
            index: i as u32,
            max_index: 30,
        });
    }
    
    // 4. Shi List (12)
    // Based on current Yun start year.
    let yun_start_ad = current.yun.start_year;
    let yun_name = &current.yun.name;
    let (yu, yl) = get_hexagram_struct(yun_name);
    
    let mut shi_list = Vec::new();
    for i in 1..=12 {
        let start = yun_start_ad + (i - 1) * 30;
        let end = start + 29;
        
        let (su, sl) = Hexagram::new(yun_name, yu, yl).change_line(((i - 1) / 2) as usize);
        let shi_gua_name = get_hexagram_name(su, sl);
        
        shi_list.push(PeriodInfo {
            name: shi_gua_name,
            start_year: start,
            end_year: end,
            index: i as u32,
            max_index: 12,
        });
    }
    
    // 5. Xun List (3)
    // Based on current Shi start year.
    let shi_start_ad = current.shi.start_year;
    let mut xun_list = Vec::new();
    for i in 1..=3 {
        let start = shi_start_ad + (i - 1) * 10;
        let end = start + 9;
        
        xun_list.push(PeriodInfo {
            name: format!("第{}旬", i),
            start_year: start,
            end_year: end,
            index: i as u32,
            max_index: 3,
        });
    }
    
    TimelineData {
        current,
        yuan_list,
        hui_list,
        yun_list,
        shi_list,
        xun_list,
    }
}
