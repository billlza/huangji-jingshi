use serde::{Deserialize, Serialize};
use chinese_lunisolar_calendar::{LunisolarDate, SolarDate};
use chrono::{Datelike, DateTime, Utc};
// use astro::*; // Unused

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LunarInfo {
    pub lunar_year: String,
    pub lunar_month: String,
    pub lunar_day: String,
    pub ganzhi_year: String,
    pub ganzhi_month: String,
    pub ganzhi_day: String,
    pub ganzhi_hour: String,
    pub zodiac: String,
    pub solar_term: Option<String>,
    pub twelve_officer: String, // 建除十二神
    pub aus_directions: String, // 吉神方位（简化）
    pub yi: Vec<String>,        // 宜
    pub ji: Vec<String>,        // 忌
}

const STEMS: [&str; 10] = ["甲", "乙", "丙", "丁", "戊", "己", "庚", "辛", "壬", "癸"];
const BRANCHES: [&str; 12] = ["子", "丑", "寅", "卯", "辰", "巳", "午", "未", "申", "酉", "戌", "亥"];
const OFFICERS: [&str; 12] = ["建", "除", "满", "平", "定", "执", "破", "危", "成", "收", "开", "闭"];

// 简单的干支计算（基于索引）
fn get_ganzhi(stem_idx: usize, branch_idx: usize) -> String {
    format!("{}{}", STEMS[stem_idx % 10], BRANCHES[branch_idx % 12])
}

// 计算日干支（基于 Julian Day）
fn get_ganzhi_day(jd: f64) -> (usize, usize, String) {
    // Adjust JD to local midnight (approximate for date calculation)
    // JD 2451545.0 is 2000-01-01 12:00:00 UTC
    // We want the integer part representing the day.
    // But Chinese day starts at 23:00 (Zi hour).
    // Let's assume input JD corresponds to the queried UTC time.
    // To get the "Day" stem/branch, we usually use the day at 00:00 or noon.
    // JD calculation:
    // 2451545.0 -> 戊午
    // Offset days = round(jd - 2451545.0) ? 
    // If JD is 2451545.2 (16:48), it is still 戊午.
    // If JD is 2451545.8 (next day 07:12), it is 己未.
    // Boundary is midnight?
    // JD days start at noon.
    // Midnight is .5
    // So 2000-01-01 00:00 is JD 2451544.5
    // 2000-01-01 23:59 is JD 2451545.49
    // So [2451544.5, 2451545.5) is 2000-01-01.
    // Let's shift JD by +0.5 to align with midnight-to-midnight days.
    // floor(jd + 0.5) should give the integer day number.
    // 2000-01-01 (noon 2451545.0) + 0.5 = 2451545.5 -> floor -> 2451545.
    // 2451545 corresponds to 戊午.
    
    let day_number = (jd + 0.5).floor() as i64;
    let epoch_day = 2451545; // 2000-01-01
    let offset = day_number - epoch_day;
    
    let stem_idx = (4 + offset).rem_euclid(10) as usize;
    let branch_idx = (6 + offset).rem_euclid(12) as usize;
    (stem_idx, branch_idx, get_ganzhi(stem_idx, branch_idx))
}

// 计算太阳黄经
fn get_solar_lambda(jd: f64) -> f64 {
    let d_jc = (jd - 2451545.0) / 36525.0;
    let l0 = 280.46646 + 36000.76983 * d_jc;
    let m = 357.52911 + 35999.05029 * d_jc;
    let m_rad = m.to_radians();
    let c = (1.914602 - 0.004817 * d_jc) * m_rad.sin()
          + 0.019993 * (2.0 * m_rad).sin();
    (l0 + c).rem_euclid(360.0)
}

// 计算节气与月建（月支）
fn get_solar_term_and_month_branch(jd: f64) -> (Option<String>, usize, String) {
    let lambda = get_solar_lambda(jd);
    
    // 315 deg = 立春 (Start of Spring) -> Month Branch 寅 (Tiger, Index 2)
    let month_idx_raw = ((lambda + 45.0) / 30.0).floor() as usize;
    let month_branch_idx = (month_idx_raw + 2) % 12;
    
    let term_names = [
        "春分", "清明", "谷雨", "立夏", "小满", "芒种",
        "夏至", "小暑", "大暑", "立秋", "处暑", "白露",
        "秋分", "寒露", "霜降", "立冬", "小雪", "大雪",
        "冬至", "小寒", "大寒", "立春", "雨水", "惊蛰"
    ];
    let term_idx = (lambda / 15.0).floor() as usize % 24;
    let current_term = term_names[term_idx].to_string();
    
    (Some(current_term), month_branch_idx, BRANCHES[month_branch_idx].to_string())
}

// 十二建除
fn get_twelve_officer(month_branch_idx: usize, day_branch_idx: usize) -> String {
    // Jian (建) is when Day Branch == Month Branch
    // Sequence: Jian, Chu, Man, Ping, Ding, Zhi, Po, Wei, Cheng, Shou, Kai, Bi
    // Index difference
    let diff = (day_branch_idx as isize - month_branch_idx as isize).rem_euclid(12) as usize;
    OFFICERS[diff].to_string()
}

fn get_yi_ji(officer: &str) -> (Vec<String>, Vec<String>) {
    let (y, j) = match officer {
        "建" => (vec!["出行", "访友", "纳财", "祭祀"], vec!["动土", "开仓", "掘井"]),
        "除" => (vec!["扫舍", "沐浴", "求医", "治病"], vec!["嫁娶", "出行", "开市"]),
        "满" => (vec!["嫁娶", "祈福", "开市", "纳财"], vec!["动土", "安葬", "破土"]),
        "平" => (vec!["修饰", "涂泥", "移徙"], vec!["入宅", "安门", "栽种"]),
        "定" => (vec!["入学", "祭祀", "裁衣", "纳畜"], vec!["词讼", "出行", "打官司"]),
        "执" => (vec!["祭祀", "祈福", "捕捉", "纳采"], vec!["开市", "出货", "移徙"]),
        "破" => (vec!["求医", "治病", "破屋", "拆卸"], vec!["嫁娶", "签约", "动土"]),
        "危" => (vec!["安床", "祭祀", "安门"], vec!["登山", "乘船", "出行"]),
        "成" => (vec!["嫁娶", "开市", "入学", "祭祀"], vec!["词讼", "打官司"]),
        "收" => (vec!["纳财", "捕捉", "索债"], vec!["放债", "出行", "安葬"]),
        "开" => (vec!["祭祀", "祈福", "入学", "开市"], vec!["安葬", "动土"]),
        "闭" => (vec!["筑堤", "安床", "补垣"], vec!["开市", "出行", "求医"]),
        _ => (vec![], vec![]),
    };
    (
        y.iter().map(|s| s.to_string()).collect(),
        j.iter().map(|s| s.to_string()).collect()
    )
}

pub fn compute_lunar(datetime: &DateTime<Utc>) -> anyhow::Result<LunarInfo> {
    let year = datetime.year() as u16;
    let month = datetime.month() as u8;
    let day = datetime.day() as u8;

    let solar_date = SolarDate::from_ymd(year, month, day)?;
    let lunar_date = LunisolarDate::from_solar_date(solar_date)?;
    
    let lunar_year = lunar_date.to_lunar_year();
    let lunar_year_str = lunar_year.to_string();
    let _lunar_year_int = lunar_year_str.parse::<u16>().unwrap_or(year);

    // 2. Ganzhi Day & JD
    let timestamp = datetime.timestamp();
    let jd = (timestamp as f64 / 86400.0) + 2440587.5;
    let (day_stem_idx, day_branch_idx, ganzhi_day) = get_ganzhi_day(jd);
    
    // 3. Solar Term & Month Branch
    let lambda = get_solar_lambda(jd);
    let (solar_term, month_branch_idx, month_branch_char) = get_solar_term_and_month_branch(jd);
    
    // 1. Ganzhi Year (Bazi Year based on LiChun 315°)
    // If current date is before LiChun (lambda in [270, 315)), it belongs to previous year.
    // Note: This simple check handles the transition around Feb 4th correctly.
    // 315 (LiChun) starts the year.
    // 270 (Winter Solstice) -> 300 (DaHan) -> 315 (LiChun)
    // So if lambda is between 270 and 315, we are in the end of the solar cycle? 
    // No, lambda resets at 0 (Spring Equinox). 
    // LiChun is 315.
    // Cycle: 0 -> ... -> 315 (LiChun) -> 360/0
    // Wait. Vernal Equinox (0) is usually March 20.
    // LiChun (315) is Feb 4.
    // So: 
    // Feb 4 (315) -> Mar 20 (0) -> ... -> Feb 4 (315)
    // If we are in [315, 360) or [0, 315), we are in the "New Year" relative to LiChun?
    // No.
    // Solar Year starts at LiChun (315).
    // So if lambda >= 315, it's the current Gregorian year's stem/branch (mostly).
    // If lambda < 315, it depends on whether we passed 0 (March 20).
    // Case 1: Jan 1 (lambda ~280) -> Before LiChun -> Previous Year.
    // Case 2: Feb 5 (lambda ~316) -> After LiChun -> Current Year.
    // Case 3: Mar 21 (lambda ~1) -> After LiChun -> Current Year.
    // Case 4: Dec 31 (lambda ~280) -> Before NEXT LiChun, but After THIS LiChun? No.
    // Lambda is cyclic 0-360.
    // Let's use year offset.
    // Generally: 
    // If month is Jan, year = gregorian - 1.
    // If month is Feb: check lambda. If lambda < 315, year = gregorian - 1.
    // Else year = gregorian.
    
    let mut bazi_year = year;
    if month == 1 {
        bazi_year = year - 1;
    } else if month == 2 && lambda < 315.0 && lambda > 270.0 {
        // 270 is Winter Solstice, safe buffer
        bazi_year = year - 1;
    }
    // For other months (3-12), bazi_year = year.
    
    // Calculate Stem/Branch for Bazi Year
    let year_stem_idx = (bazi_year as isize - 4).rem_euclid(10) as usize;
    let year_branch_idx = (bazi_year as isize - 4).rem_euclid(12) as usize;
    let ganzhi_year = get_ganzhi(year_stem_idx, year_branch_idx);
    
    // 4. Ganzhi Month (Five Tigers)
    // Formula: (YearStem%5 * 2 + 2) % 10 -> Stem of Month 2 (寅)
    let first_month_stem_idx = (year_stem_idx % 5 * 2 + 2) % 10;
    let month_offset = (month_branch_idx as isize - 2 + 12).rem_euclid(12) as usize;
    let current_month_stem_idx = (first_month_stem_idx + month_offset) % 10;
    let ganzhi_month = format!("{}{}", STEMS[current_month_stem_idx], month_branch_char);

    // 5. Ganzhi Hour (Five Rats)
    // Formula: (DayStem%5 * 2 + HourBranch) % 10
    // Hour Branch: (H+1)/2 % 12 (traditional formula)
    use chrono::Timelike;
    let hour = datetime.hour();
    #[allow(clippy::manual_div_ceil)]
    let hour_branch_idx = ((hour + 1) / 2 % 24 % 12) as usize;
    let hour_stem_idx = (day_stem_idx % 5 * 2 + hour_branch_idx) % 10;
    let ganzhi_hour = get_ganzhi(hour_stem_idx, hour_branch_idx);

    // 6. Twelve Officer
    let officer = get_twelve_officer(month_branch_idx, day_branch_idx);
    let (yi, ji) = get_yi_ji(&officer);

    // Lunar month string
    let lunar_month_val = lunar_date.to_lunar_month();
    // to_string() might return "十月". If we want just "十", we might need to trim "月".
    // Or in frontend we don't add "月".
    // Let's assume we keep it as is here, and fix frontend.
    // But wait, "十月" is standard.
    // If `to_string()` returns "十月", then `lunar_month_str` is "十月".
    let lunar_month_str = lunar_month_val.to_string();
    
    Ok(LunarInfo {
        lunar_year: lunar_year.to_string(),
        lunar_month: lunar_month_str,
        lunar_day: lunar_date.to_lunar_day().to_string(),
        ganzhi_year,
        ganzhi_month, 
        ganzhi_day,
        ganzhi_hour,
        zodiac: lunar_year.to_zodiac().to_string(),
        solar_term, 
        twelve_officer: officer, 
        aus_directions: "喜神: 东北, 财神: 正北".to_string(),
        yi,
        ji,
    })
}
