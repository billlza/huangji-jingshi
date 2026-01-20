use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use crate::{data, lunar, algorithm};
use crate::calendar::time_rule::{utc_to_hj_year, YearStartMode};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FortuneRequest {
    pub datetime: DateTime<Utc>,
    /// 时区偏移（分钟），东为正 UTC+8=+480, 西为负 UTC-5=-300
    #[serde(default)]
    pub tz_offset_minutes: Option<i32>,
    /// 经度（用于真太阳时校正）
    #[serde(default)]
    pub lon: Option<f64>,
    /// 是否使用真太阳时（默认 false）
    #[serde(default)]
    pub use_true_solar_time: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FortuneResponse {
    // 皇极经世
    pub yuan: String,
    pub hui: String,
    pub yun: String,
    pub shi: String,
    pub xun: String,
    pub nian_ganzhi: String,
    pub hexagram_major: String,
    pub hexagram_minor: Option<String>,
    pub hexagram_code: Option<Vec<u8>>, // New: Binary structure [top, ..., bottom] (0/1)
    pub flying_star: Option<String>,    // New: Annual Flying Star (e.g. "二黑")
    
    // Period Info for Timeline
    pub period_info: Option<algorithm::HuangjiInfo>,

    // Critical points (next starts)
    pub next_yun_start_year: Option<i32>,
    pub next_shi_start_year: Option<i32>,
    pub next_xun_start_year: Option<i32>,

    // 农历 / 黄历
    pub lunar: Option<lunar::LunarInfo>,
    
    pub note: String,
    pub mapping_record: Option<crate::data::YearRecord>,
}

pub fn compute_fortune(req: &FortuneRequest) -> FortuneResponse {
    // 用统一时间规则把 UTC 转换为经世年（无公元0年），避免 UTC 跨年导致的年界错误
    let tz_offset_minutes = req.tz_offset_minutes.unwrap_or(480);
    let lon = req.lon.unwrap_or(116.4);
    let use_true_solar_time = req.use_true_solar_time.unwrap_or(false);
    let year = utc_to_hj_year(
        req.datetime,
        tz_offset_minutes,
        lon,
        use_true_solar_time,
        YearStartMode::GregorianNewYear,
    );
    
    // 1. Calculate Algorithmically (High Priority for Fortune)
    let algo_info = algorithm::get_hj_info(year);
    
    // 2. Fetch Data for Dynasty/Person (Supplemental)
    let (dynasty_info, _xun_info, mapping_record) = if let Some(record) = data::get_year_record(year) {
        let note = format!("{} {}", record.dynasty, record.person).trim().to_string();
        let xun = record.xun_raw.replace("旬", "").trim().to_string();
        (note, xun, Some(record))
    } else {
        (format!("数据未覆盖 {} 年", year), "?".to_string(), None)
    };
    
    // 3. Calculate Lunar/Ganzhi（按规则时间取本地年月日/时）
    let lunar_info = lunar::compute_lunar(&req.datetime, tz_offset_minutes, lon, use_true_solar_time).ok();
    let ganzhi = lunar_info.as_ref().map(|l| l.ganzhi_year.clone()).unwrap_or_else(|| "未知".to_string());

    // 4. Calculate Hexagram Code (Binary)
    // Structure: (upper, lower). Each is 3-bit integer.
    // We want array of 6 bits, from TOP to BOTTOM.
    // Upper: bit 2, 1, 0. Lower: bit 2, 1, 0.
    let (u, l) = algorithm::get_hexagram_struct(&algo_info.year_gua);
    // Upper trigram bits (2,1,0), then Lower trigram bits (2,1,0)
    let hex_code = vec![
        (u >> 2) & 1,
        (u >> 1) & 1,
        u & 1,
        (l >> 2) & 1,
        (l >> 1) & 1,
        l & 1,
    ];

    // 5. Calculate Annual Flying Star (Nine Palace)
    let mut star_val = (11 - (if year > 0 { year } else { year + 1 }) % 9) % 9;
    if star_val == 0 { star_val = 9; }
    if star_val < 0 { star_val += 9; } 
    
    let stars = ["", "一白贪狼", "二黑巨门", "三碧禄存", "四绿文曲", "五黄廉贞", "六白武曲", "七赤破军", "八白左辅", "九紫右弼"];
    let flying_star = if (1..=9).contains(&star_val) {
        stars[star_val as usize].to_string()
    } else {
        String::new()
    };

    let next_yun = algo_info.yun.end_year + 1;
    let next_shi = algo_info.shi.end_year + 1;
    let next_xun = algo_info.xun.end_year + 1;

    FortuneResponse {
        yuan: algo_info.yuan.name.clone(),
        hui: algo_info.hui.name.clone(),
        yun: algo_info.yun.name.clone(),
        shi: algo_info.shi.name.clone(),
        xun: algo_info.xun.name.clone(),
        nian_ganzhi: ganzhi,
        hexagram_major: algo_info.year_gua.clone(),
        hexagram_minor: None,
        hexagram_code: Some(hex_code),
        flying_star: Some(flying_star),
        period_info: Some(algo_info),
        lunar: lunar_info,
        note: dynasty_info,
        mapping_record,
        next_yun_start_year: Some(next_yun),
        next_shi_start_year: Some(next_shi),
        next_xun_start_year: Some(next_xun),
    }
}
