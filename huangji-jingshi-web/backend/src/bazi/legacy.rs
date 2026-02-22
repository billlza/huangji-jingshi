use crate::bazi::models::{BaziRequestContext, DayRollover, PillarIndices, TimeBasis};
use chrono::{Datelike, Timelike};
use huangji_core::astro::solar::{
    datetime_to_jd, hour_to_dizhi_index, solar_position, true_solar_hour, true_solar_time,
    utc_to_jd,
};
use huangji_core::calendar::ganzhi::{
    calc_day_pillar, calc_dayun_start_age, calc_hour_pillar, calc_month_pillar, calc_year_pillar,
    DIZHI, GAN_WUXING, NAYIN, SHENGXIAO, TIANGAN, ZHI_WUXING,
};
use huangji_core::calendar::jieqi::SolarTerm;
use serde_json::{json, Value};
use std::collections::HashMap;

const ZHI_CANGGAN: [[&str; 3]; 12] = [
    ["", "", "癸"],
    ["癸", "辛", "己"],
    ["戊", "丙", "甲"],
    ["", "", "乙"],
    ["癸", "乙", "戊"],
    ["戊", "庚", "丙"],
    ["己", "", "丁"],
    ["丁", "乙", "己"],
    ["戊", "壬", "庚"],
    ["", "", "辛"],
    ["丁", "辛", "戊"],
    ["甲", "", "壬"],
];

#[allow(clippy::manual_is_multiple_of)]
fn calculate_ten_god(day_gan_idx: usize, target_gan_idx: usize) -> &'static str {
    let day_is_yang = day_gan_idx % 2 == 0;
    let target_is_yang = target_gan_idx % 2 == 0;
    let same_yin_yang = day_is_yang == target_is_yang;

    let day_wuxing = day_gan_idx / 2;
    let target_wuxing = target_gan_idx / 2;
    let relation = (target_wuxing + 5 - day_wuxing) % 5;

    match relation {
        0 => {
            if same_yin_yang {
                "比肩"
            } else {
                "劫财"
            }
        }
        1 => {
            if same_yin_yang {
                "食神"
            } else {
                "伤官"
            }
        }
        2 => {
            if same_yin_yang {
                "偏财"
            } else {
                "正财"
            }
        }
        3 => {
            if same_yin_yang {
                "偏官"
            } else {
                "正官"
            }
        }
        4 => {
            if same_yin_yang {
                "偏印"
            } else {
                "正印"
            }
        }
        _ => "未知",
    }
}

fn get_hidden_stems_with_gods(zhi_idx: usize, day_gan_idx: usize) -> Vec<Value> {
    let hidden = &ZHI_CANGGAN[zhi_idx];
    let mut result = Vec::new();

    for (i, gan_str) in hidden.iter().enumerate() {
        if gan_str.is_empty() {
            continue;
        }
        if let Some(gan_idx) = TIANGAN.iter().position(|&g| g == *gan_str) {
            let ten_god = calculate_ten_god(day_gan_idx, gan_idx);
            let gan_wuxing = GAN_WUXING[gan_idx];
            let (canggan_type, energy) = match i {
                0 => ("余气", 30),
                1 => {
                    if hidden[0].is_empty() {
                        ("余气", 30)
                    } else {
                        ("中气", 20)
                    }
                }
                2 => ("本气", 50),
                _ => ("", 0),
            };

            result.push(json!({
                "gan": gan_str,
                "gan_wuxing": gan_wuxing,
                "ten_god": ten_god,
                "type": canggan_type,
                "energy": energy
            }));
        }
    }

    result
}

fn calculate_dayun(
    month_gan_idx: i32,
    month_zhi_idx: i32,
    year_gan_idx: i32,
    gender: &str,
    birth_year: i32,
    start_age: f64,
) -> Vec<Value> {
    let year_is_yang = year_gan_idx % 2 == 0;
    let forward = (gender == "male" && year_is_yang) || (gender == "female" && !year_is_yang);
    let mut dayun_cycles = Vec::new();

    for i in 0..10 {
        let cycle_num = if forward { i + 1 } else { -(i + 1) };
        let gan_idx = ((month_gan_idx + cycle_num + 10) % 10 + 10) % 10;
        let zhi_idx = ((month_zhi_idx + cycle_num + 12) % 12 + 12) % 12;

        let start_age_for_cycle = start_age + (i as f64 * 10.0);
        let end_age = start_age_for_cycle + 9.0;

        dayun_cycles.push(json!({
            "cycle": i + 1,
            "gan": TIANGAN[gan_idx as usize],
            "zhi": DIZHI[zhi_idx as usize],
            "gan_wuxing": GAN_WUXING[gan_idx as usize],
            "zhi_wuxing": ZHI_WUXING[zhi_idx as usize],
            "start_age": start_age_for_cycle.round() as i32,
            "end_age": end_age.round() as i32,
            "year_range": format!(
                "{}-{}",
                birth_year + start_age_for_cycle.round() as i32,
                birth_year + end_age.round() as i32
            )
        }));
    }

    dayun_cycles
}

fn calculate_xiaoyun(
    hour_gan_idx: i32,
    hour_zhi_idx: i32,
    gender: &str,
    birth_year: i32,
    current_year: i32,
) -> Value {
    let forward = gender == "male";
    let age = current_year - birth_year;
    let offset = if forward { age } else { -age };
    let gan_idx = ((hour_gan_idx + offset + 10) % 10 + 10) % 10;
    let zhi_idx = ((hour_zhi_idx + offset + 12) % 12 + 12) % 12;

    json!({
        "age": age,
        "year": current_year,
        "gan": TIANGAN[gan_idx as usize],
        "zhi": DIZHI[zhi_idx as usize],
        "gan_wuxing": GAN_WUXING[gan_idx as usize],
        "zhi_wuxing": ZHI_WUXING[zhi_idx as usize]
    })
}

fn calculate_liunian(birth_year: i32, current_year: i32, num_years: i32) -> Vec<Value> {
    let mut liunian = Vec::new();
    for i in 0..num_years {
        let year = current_year + i;
        let age = year - birth_year;
        let gan_idx = ((year - 4) % 10 + 10) % 10;
        let zhi_idx = ((year - 4) % 12 + 12) % 12;
        liunian.push(json!({
            "year": year,
            "age": age,
            "gan": TIANGAN[gan_idx as usize],
            "zhi": DIZHI[zhi_idx as usize],
            "gan_wuxing": GAN_WUXING[gan_idx as usize],
            "zhi_wuxing": ZHI_WUXING[zhi_idx as usize],
            "zodiac": SHENGXIAO[zhi_idx as usize]
        }));
    }
    liunian
}

fn basis_local_datetime(ctx: &BaziRequestContext) -> chrono::NaiveDateTime {
    let standard_local = ctx.zone.local_datetime(ctx.datetime_utc);
    if matches!(ctx.time_basis, TimeBasis::Standard) {
        return standard_local;
    }

    let standard_meridian = (ctx.tz_offset_minutes as f64 / 60.0) * 15.0;
    true_solar_time(&ctx.datetime_utc, ctx.longitude, standard_meridian)
}

pub fn compute_legacy_pillars(ctx: &BaziRequestContext) -> PillarIndices {
    let jd_utc = utc_to_jd(&ctx.datetime_utc);
    let solar_longitude = solar_position(jd_utc).ecliptic_longitude;
    let basis_local = basis_local_datetime(ctx);

    let basis_hour = basis_local.hour() as f64
        + basis_local.minute() as f64 / 60.0
        + basis_local.second() as f64 / 3600.0;
    let (hour_zhi_idx, default_late_zi) = hour_to_dizhi_index(basis_hour.rem_euclid(24.0));
    let is_late_zi = match ctx.day_rollover {
        DayRollover::ZiChu23 => default_late_zi,
        DayRollover::ZiZheng00 => false,
    };

    let basis_jd = datetime_to_jd(&basis_local);
    let day_pillar = calc_day_pillar(basis_jd, is_late_zi);
    let year_pillar = calc_year_pillar(basis_local.year(), solar_longitude);
    let month_pillar = calc_month_pillar(year_pillar.0, solar_longitude);
    let hour_pillar = calc_hour_pillar(day_pillar.0, hour_zhi_idx);

    PillarIndices {
        year: year_pillar,
        month: month_pillar,
        day: day_pillar,
        hour: hour_pillar,
        solar_longitude: Some(solar_longitude),
        solar_term: Some(
            SolarTerm::from_longitude(solar_longitude)
                .name()
                .to_string(),
        ),
        true_solar_hour: Some(true_solar_hour(&ctx.datetime_utc, ctx.longitude)),
        is_late_zi,
    }
}

pub fn build_payload_from_pillars(ctx: &BaziRequestContext, pillars: &PillarIndices) -> Value {
    let year_gan_idx = pillars.year.0 as i32;
    let year_zhi_idx = pillars.year.1 as i32;
    let month_gan_idx = pillars.month.0 as i32;
    let month_zhi_idx = pillars.month.1 as i32;
    let day_gan_idx = pillars.day.0 as i32;
    let day_zhi_idx = pillars.day.1 as i32;
    let hour_gan_idx = pillars.hour.0 as i32;
    let hour_zhi_idx = pillars.hour.1 as i32;

    let create_pillar = |gan_idx: i32, zhi_idx: i32, day_gan_idx: usize| -> Value {
        let gi = gan_idx as usize % 10;
        let zi = zhi_idx as usize % 12;
        let nayin_idx = ((gi / 2) * 6 + zi / 2) % 30;
        let gan_ten_god = calculate_ten_god(day_gan_idx, gi);
        let hidden_stems = get_hidden_stems_with_gods(zi, day_gan_idx);

        json!({
            "gan": TIANGAN[gi],
            "zhi": DIZHI[zi],
            "gan_wuxing": GAN_WUXING[gi],
            "zhi_wuxing": ZHI_WUXING[zi],
            "zhi_animal": SHENGXIAO[zi],
            "nayin": NAYIN[nayin_idx],
            "gan_ten_god": gan_ten_god,
            "hidden_stems": hidden_stems
        })
    };

    let day_gan_idx_usize = day_gan_idx as usize % 10;
    let year_pillar = create_pillar(year_gan_idx, year_zhi_idx, day_gan_idx_usize);
    let month_pillar = create_pillar(month_gan_idx, month_zhi_idx, day_gan_idx_usize);
    let day_pillar = create_pillar(day_gan_idx, day_zhi_idx, day_gan_idx_usize);
    let hour_pillar = create_pillar(hour_gan_idx, hour_zhi_idx, day_gan_idx_usize);

    let mut wuxing_counts: HashMap<String, i32> = HashMap::new();
    wuxing_counts.insert("木".to_string(), 0);
    wuxing_counts.insert("火".to_string(), 0);
    wuxing_counts.insert("土".to_string(), 0);
    wuxing_counts.insert("金".to_string(), 0);
    wuxing_counts.insert("水".to_string(), 0);

    for idx in [year_gan_idx, month_gan_idx, day_gan_idx, hour_gan_idx] {
        let wx = GAN_WUXING[idx as usize % 10]
            .replace("阳", "")
            .replace("阴", "");
        *wuxing_counts.entry(wx).or_insert(0) += 1;
    }
    for idx in [year_zhi_idx, month_zhi_idx, day_zhi_idx, hour_zhi_idx] {
        let wx = ZHI_WUXING[idx as usize % 12]
            .replace("阳", "")
            .replace("阴", "");
        *wuxing_counts.entry(wx).or_insert(0) += 1;
    }

    let day_master = GAN_WUXING[day_gan_idx as usize % 10];
    let day_master_wx = day_master.replace("阳", "").replace("阴", "");
    let day_master_count = wuxing_counts.get(&day_master_wx).unwrap_or(&0);
    let strength = if *day_master_count >= 3 {
        "strong"
    } else if *day_master_count <= 1 {
        "weak"
    } else {
        "balanced"
    };

    let missing: Vec<&str> = ["木", "火", "土", "金", "水"]
        .iter()
        .filter(|wx| *wuxing_counts.get(**wx).unwrap_or(&0) == 0)
        .copied()
        .collect();

    let jd_for_dayun = datetime_to_jd(&basis_local_datetime(ctx));
    let is_male = ctx.gender == "male";
    let start_age = calc_dayun_start_age(jd_for_dayun, pillars.year.0, is_male);
    let birth_year = ctx.birth_year();
    let current_year = ctx.current_year();

    let dayun = calculate_dayun(
        month_gan_idx,
        month_zhi_idx,
        year_gan_idx,
        &ctx.gender,
        birth_year,
        start_age,
    );
    let xiaoyun = calculate_xiaoyun(
        hour_gan_idx,
        hour_zhi_idx,
        &ctx.gender,
        birth_year,
        current_year,
    );
    let liunian = calculate_liunian(birth_year, current_year, 6);
    let day_gan_str = TIANGAN[day_gan_idx as usize % 10];

    json!({
        "year_pillar": year_pillar,
        "month_pillar": month_pillar,
        "day_pillar": day_pillar,
        "hour_pillar": hour_pillar,
        "wuxing_analysis": {
            "day_master": day_master,
            "day_master_gan": day_gan_str,
            "day_master_strength": strength,
            "wuxing_counts": wuxing_counts,
            "missing_wuxing": missing
        },
        "ten_gods_summary": {
            "year_gan": year_pillar["gan_ten_god"],
            "month_gan": month_pillar["gan_ten_god"],
            "day_gan": day_pillar["gan_ten_god"],
            "hour_gan": hour_pillar["gan_ten_god"]
        },
        "dayun": dayun,
        "xiaoyun": xiaoyun,
        "liunian": liunian,
        "gender": ctx.gender,
        "birth_year": birth_year,
        "zodiac": SHENGXIAO[year_zhi_idx as usize % 12],
        "solar_term": pillars.solar_term.clone().unwrap_or_else(|| "未知".to_string()),
        "start_age": start_age.round() as i32,
        "solar_longitude": pillars.solar_longitude,
        "true_solar_hour": pillars.true_solar_hour,
        "is_late_zi": pillars.is_late_zi,
        "longitude": ctx.longitude,
        "tz_offset_minutes": ctx.tz_offset_minutes,
        "time_basis": ctx.time_basis.as_str(),
        "day_rollover": ctx.day_rollover.as_str()
    })
}

pub fn build_legacy_payload(ctx: &BaziRequestContext) -> Value {
    let pillars = compute_legacy_pillars(ctx);
    build_payload_from_pillars(ctx, &pillars)
}
