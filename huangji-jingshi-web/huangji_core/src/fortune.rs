use crate::calendar::time_rule::{utc_to_hj_year, YearStartMode};
use crate::{algorithm, huangji_table, lunar, table_engine};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::str::FromStr;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "lowercase")]
pub enum CalcMode {
    Algorithm,
    Table,
    #[default]
    Compare,
}

impl FromStr for CalcMode {
    type Err = ();

    fn from_str(input: &str) -> Result<Self, Self::Err> {
        match input.trim().to_ascii_lowercase().as_str() {
            "algorithm" => Ok(Self::Algorithm),
            "table" => Ok(Self::Table),
            "compare" => Ok(Self::Compare),
            _ => Err(()),
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "lowercase")]
pub enum PrimaryMode {
    #[default]
    Algorithm,
    Table,
}

impl FromStr for PrimaryMode {
    type Err = ();

    fn from_str(input: &str) -> Result<Self, Self::Err> {
        match input.trim().to_ascii_lowercase().as_str() {
            "algorithm" => Ok(Self::Algorithm),
            "table" => Ok(Self::Table),
            _ => Err(()),
        }
    }
}

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
    /// 计算模式：algorithm|table|compare（默认 compare）
    #[serde(default)]
    pub mode: Option<CalcMode>,
    /// 年界模式：lichun|gregorian（默认 lichun）
    #[serde(default)]
    pub year_start: Option<YearStartMode>,
    /// compare 模式下主值来源（默认 algorithm）
    #[serde(default)]
    pub primary: Option<PrimaryMode>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FortuneVariant {
    pub source: String,
    pub available: bool,
    pub yuan: String,
    pub hui: String,
    pub yun: String,
    pub shi: String,
    pub xun: String,
    pub nian_ganzhi: String,
    pub hexagram_major: String,
    pub note: String,
    pub period_info: Option<algorithm::HuangjiInfo>,
    pub mapping_record: Option<huangji_table::YearRecord>,
    pub mapping_record_normalized: Option<huangji_table::NormalizedYearRecord>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FortuneVariants {
    pub algorithm: FortuneVariant,
    pub table_raw: FortuneVariant,
    pub table_normalized: FortuneVariant,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FortuneCalcMeta {
    pub mode: CalcMode,
    pub primary: PrimaryMode,
    pub year_start: String,
    pub hj_year: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FortuneDiff {
    pub hexagram_major_diff: bool,
    pub yun_diff: bool,
    pub shi_diff: bool,
    pub xun_diff: bool,
    pub note: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthorityEvidenceRef {
    pub label: String,
    pub url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthorityCoverage {
    pub min_year: i32,
    pub max_year: i32,
    pub covered: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthorityMeta {
    pub requested_source: String,
    pub resolved_source: String,
    pub table_coverage: Option<AuthorityCoverage>,
    pub fallback_reason: Option<String>,
    pub authority_level: String,
    pub evidence_refs: Vec<AuthorityEvidenceRef>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FortuneResponse {
    // 皇极经世主输出（兼容旧字段）
    pub yuan: String,
    pub hui: String,
    pub yun: String,
    pub shi: String,
    pub xun: String,
    pub nian_ganzhi: String,
    pub hexagram_major: String,
    pub hexagram_minor: Option<String>,
    pub hexagram_code: Option<Vec<u8>>,
    pub flying_star: Option<String>,

    // Period Info for Timeline
    pub period_info: Option<algorithm::HuangjiInfo>,

    // Critical points (next starts)
    pub next_yun_start_year: Option<i32>,
    pub next_shi_start_year: Option<i32>,
    pub next_xun_start_year: Option<i32>,

    // 农历 / 黄历
    pub lunar: Option<lunar::LunarInfo>,

    pub note: String,
    pub mapping_record: Option<huangji_table::YearRecord>,

    // 新增元信息
    pub calc_meta: Option<FortuneCalcMeta>,
    pub variants: Option<FortuneVariants>,
    pub diff: Option<FortuneDiff>,
    pub authority: Option<AuthorityMeta>,
}

fn year_start_label(mode: YearStartMode) -> &'static str {
    match mode {
        YearStartMode::Lichun => "lichun",
        YearStartMode::GregorianNewYear => "gregorian",
    }
}

pub fn requires_table_source(mode: CalcMode, primary: PrimaryMode) -> bool {
    matches!(mode, CalcMode::Table)
        || (matches!(mode, CalcMode::Compare) && matches!(primary, PrimaryMode::Table))
}

fn requested_source_primary(mode: CalcMode, primary: PrimaryMode) -> PrimaryMode {
    match mode {
        CalcMode::Algorithm => PrimaryMode::Algorithm,
        CalcMode::Table => PrimaryMode::Table,
        CalcMode::Compare => primary,
    }
}

fn evidence_refs() -> Vec<AuthorityEvidenceRef> {
    vec![
        AuthorityEvidenceRef {
            label: "皇極經世".to_string(),
            url: "https://zh.wikisource.org/zh-hant/%E7%9A%87%E6%A5%B5%E7%B6%93%E4%B8%96".to_string(),
        },
        AuthorityEvidenceRef {
            label: "易學象數論/皇極一".to_string(),
            url: "https://zh.wikisource.org/zh-hant/%E6%98%93%E5%AD%B8%E8%B1%A1%E6%95%B8%E8%AB%96/%E7%9A%87%E6%A5%B5%E4%B8%80".to_string(),
        },
        AuthorityEvidenceRef {
            label: "易學象數論/皇極二".to_string(),
            url: "https://zh.wikisource.org/zh-hant/%E6%98%93%E5%AD%B8%E8%B1%A1%E6%95%B8%E8%AB%96/%E7%9A%87%E6%A5%B5%E4%BA%8C".to_string(),
        },
        AuthorityEvidenceRef {
            label: "觀物外篇衍義 卷3".to_string(),
            url: "https://zh.wikisource.org/zh-hant/%E8%A7%80%E7%89%A9%E5%A4%96%E7%AF%87%E8%A1%8D%E7%BE%A9%E5%8D%B73".to_string(),
        },
        AuthorityEvidenceRef {
            label: "year_mapping_canonical.json".to_string(),
            url: "repo:/huangji_core/data/year_mapping_canonical.json".to_string(),
        },
    ]
}

fn resolve_note(record: &Option<huangji_table::YearRecord>, year: i32) -> String {
    if let Some(record) = record {
        let note = format!("{} {}", record.dynasty.trim(), record.person.trim())
            .trim()
            .to_string();
        if note.is_empty() {
            format!("年表已命中 {} 年", year)
        } else {
            note
        }
    } else if let Some(coverage) = huangji_table::get_coverage() {
        format!(
            "年表未覆盖 {} 年（覆盖范围: {}-{}）",
            year, coverage.min_year, coverage.max_year
        )
    } else {
        format!("年表未覆盖 {} 年", year)
    }
}

fn build_variant_from_algorithm(
    source: &str,
    year: i32,
    algo_info: &algorithm::HuangjiInfo,
    nian_ganzhi: &str,
    mapping_record: Option<huangji_table::YearRecord>,
    mapping_normalized: Option<huangji_table::NormalizedYearRecord>,
) -> FortuneVariant {
    FortuneVariant {
        source: source.to_string(),
        available: true,
        yuan: algo_info.yuan.name.clone(),
        hui: algo_info.hui.name.clone(),
        yun: algo_info.yun.name.clone(),
        shi: algo_info.shi.name.clone(),
        xun: algo_info.xun.name.clone(),
        nian_ganzhi: nian_ganzhi.to_string(),
        hexagram_major: algo_info.year_gua.clone(),
        note: resolve_note(&mapping_record, year),
        period_info: Some(algo_info.clone()),
        mapping_record,
        mapping_record_normalized: mapping_normalized,
    }
}

fn project_table_raw(base: &FortuneVariant) -> FortuneVariant {
    let mut result = FortuneVariant {
        source: "table_raw".to_string(),
        available: false,
        yuan: "未载".to_string(),
        hui: "未载".to_string(),
        yun: "未载".to_string(),
        shi: "未载".to_string(),
        xun: "未载".to_string(),
        nian_ganzhi: "未载".to_string(),
        hexagram_major: "未载".to_string(),
        note: "table_raw unavailable for this year".to_string(),
        period_info: None,
        mapping_record: base.mapping_record.clone(),
        mapping_record_normalized: base.mapping_record_normalized.clone(),
    };

    if let Some(record) = &result.mapping_record {
        result.available = true;
        result.hexagram_major = record.nian_hexagram.clone();
        result.nian_ganzhi = if record.ganzhi.trim().is_empty() {
            base.nian_ganzhi.clone()
        } else {
            record.ganzhi.trim().to_string()
        };
        if let Some(normalized) = &result.mapping_record_normalized {
            if let Some(hui_name) = &normalized.hui_name {
                result.hui = hui_name.clone();
            }
            if let Some(yun_name) = &normalized.yun_name {
                result.yun = yun_name.clone();
            }
            if let Some(shi_name) = &normalized.shi_name {
                result.shi = shi_name.clone();
            }
            if let Some(xun_name) = &normalized.xun_name {
                result.xun = xun_name.clone();
            }
        }
        result.note = resolve_note(&Some(record.clone()), record.gregorian_year);
    }
    result
}

fn project_table_canonical(base: &FortuneVariant, year: i32) -> FortuneVariant {
    let coverage = table_engine::get_coverage();
    let mut result = FortuneVariant {
        source: "table_normalized".to_string(),
        available: false,
        yuan: "未载".to_string(),
        hui: "未载".to_string(),
        yun: "未载".to_string(),
        shi: "未载".to_string(),
        xun: "未载".to_string(),
        nian_ganzhi: "未载".to_string(),
        hexagram_major: "未载".to_string(),
        note: coverage.map_or_else(
            || "table_canonical unavailable".to_string(),
            |range| {
                format!(
                    "table_canonical unavailable for {} (coverage: {}-{})",
                    year, range.min_year, range.max_year
                )
            },
        ),
        period_info: None,
        mapping_record: base.mapping_record.clone(),
        mapping_record_normalized: base.mapping_record_normalized.clone(),
    };

    if let Some(record) = table_engine::get_year_record(year) {
        result.available = true;
        result.yuan = record.yuan_name;
        result.hui = record.hui_name;
        result.yun = record.yun_name;
        result.shi = record.shi_name;
        result.xun = record.xun_name;
        result.nian_ganzhi = if record.ganzhi.trim().is_empty() {
            base.nian_ganzhi.clone()
        } else {
            record.ganzhi
        };
        result.hexagram_major = record.year_hexagram;
        result.period_info = table_engine::get_hj_info(year);
        result.note = "canonical table projection".to_string();
    }

    result
}

fn select_variant<'a>(
    mode: CalcMode,
    requested_primary: PrimaryMode,
    algorithm_variant: &'a FortuneVariant,
    table_variant: &'a FortuneVariant,
) -> (&'a FortuneVariant, PrimaryMode) {
    match requested_source_primary(mode, requested_primary) {
        PrimaryMode::Algorithm => (algorithm_variant, PrimaryMode::Algorithm),
        PrimaryMode::Table => {
            if table_variant.available {
                (table_variant, PrimaryMode::Table)
            } else {
                (algorithm_variant, PrimaryMode::Algorithm)
            }
        }
    }
}

fn calc_hexagram_code(hexagram_name: &str) -> Option<Vec<u8>> {
    let canonical = huangji_table::normalize_hexagram_name(hexagram_name)
        .unwrap_or_else(|| hexagram_name.to_string());
    let (u, l) = algorithm::get_hexagram_struct(&canonical);
    if canonical != "坤" && (u, l) == (0, 0) {
        return None;
    }
    Some(vec![
        (u >> 2) & 1,
        (u >> 1) & 1,
        u & 1,
        (l >> 2) & 1,
        (l >> 1) & 1,
        l & 1,
    ])
}

fn calc_flying_star(year: i32) -> String {
    let mut star_val = (11 - (if year > 0 { year } else { year + 1 }) % 9) % 9;
    if star_val == 0 {
        star_val = 9;
    }
    if star_val < 0 {
        star_val += 9;
    }
    let stars = [
        "",
        "一白贪狼",
        "二黑巨门",
        "三碧禄存",
        "四绿文曲",
        "五黄廉贞",
        "六白武曲",
        "七赤破军",
        "八白左辅",
        "九紫右弼",
    ];
    stars
        .get(star_val as usize)
        .copied()
        .unwrap_or("")
        .to_string()
}

pub fn compute_fortune(req: &FortuneRequest) -> FortuneResponse {
    let mode = req.mode.unwrap_or_default();
    let requested_primary_mode = req.primary.unwrap_or_default();
    let year_start = req.year_start.unwrap_or_default();

    // 用统一时间规则把 UTC 转换为经世年（无公元0年）
    let tz_offset_minutes = req.tz_offset_minutes.unwrap_or(480);
    let lon = req.lon.unwrap_or(116.4);
    let use_true_solar_time = req.use_true_solar_time.unwrap_or(false);
    let year = utc_to_hj_year(
        req.datetime,
        tz_offset_minutes,
        lon,
        use_true_solar_time,
        year_start,
    );

    let algo_info = algorithm::get_hj_info(year);
    let mapping_record = huangji_table::get_year_record(year);
    let mapping_record_normalized = huangji_table::get_year_record_normalized(year);

    let lunar_info =
        lunar::compute_lunar(&req.datetime, tz_offset_minutes, lon, use_true_solar_time).ok();
    let ganzhi = lunar_info
        .as_ref()
        .map(|l| l.ganzhi_year.clone())
        .unwrap_or_else(|| "未知".to_string());

    let algorithm_variant = build_variant_from_algorithm(
        "algorithm",
        year,
        &algo_info,
        &ganzhi,
        mapping_record.clone(),
        mapping_record_normalized.clone(),
    );
    let table_raw_variant = project_table_raw(&algorithm_variant);
    let table_normalized_variant = project_table_canonical(&algorithm_variant, year);

    let (selected, resolved_primary) = select_variant(
        mode,
        requested_primary_mode,
        &algorithm_variant,
        &table_normalized_variant,
    );
    let requested_source = requested_source_primary(mode, requested_primary_mode);

    let coverage = table_engine::get_coverage();
    let table_coverage = coverage.as_ref().map(|range| AuthorityCoverage {
        min_year: range.min_year,
        max_year: range.max_year,
        covered: year >= range.min_year && year <= range.max_year,
    });
    let fallback_reason =
        if matches!(requested_source, PrimaryMode::Table) && !table_normalized_variant.available {
            Some("table_not_covered".to_string())
        } else {
            None
        };
    let authority_level = if matches!(resolved_primary, PrimaryMode::Table) {
        "canonical".to_string()
    } else {
        "derived".to_string()
    };
    let authority = AuthorityMeta {
        requested_source: if matches!(requested_source, PrimaryMode::Table) {
            "table".to_string()
        } else {
            "algorithm".to_string()
        },
        resolved_source: if matches!(resolved_primary, PrimaryMode::Table) {
            "table".to_string()
        } else {
            "algorithm".to_string()
        },
        table_coverage,
        fallback_reason,
        authority_level,
        evidence_refs: evidence_refs(),
    };

    let next_yun = selected
        .period_info
        .as_ref()
        .map(|info| info.yun.end_year + 1);
    let next_shi = selected
        .period_info
        .as_ref()
        .map(|info| info.shi.end_year + 1);
    let next_xun = selected
        .period_info
        .as_ref()
        .map(|info| info.xun.end_year + 1);

    let diff = if table_normalized_variant.available {
        FortuneDiff {
            hexagram_major_diff: algorithm_variant.hexagram_major
                != table_normalized_variant.hexagram_major,
            yun_diff: algorithm_variant.yun != table_normalized_variant.yun,
            shi_diff: algorithm_variant.shi != table_normalized_variant.shi,
            xun_diff: algorithm_variant.xun != table_normalized_variant.xun,
            note: "algorithm vs canonical_table".to_string(),
        }
    } else {
        FortuneDiff {
            hexagram_major_diff: false,
            yun_diff: false,
            shi_diff: false,
            xun_diff: false,
            note: "canonical_table unavailable for this year".to_string(),
        }
    };

    FortuneResponse {
        yuan: selected.yuan.clone(),
        hui: selected.hui.clone(),
        yun: selected.yun.clone(),
        shi: selected.shi.clone(),
        xun: selected.xun.clone(),
        nian_ganzhi: selected.nian_ganzhi.clone(),
        hexagram_major: selected.hexagram_major.clone(),
        hexagram_minor: None,
        hexagram_code: calc_hexagram_code(&selected.hexagram_major),
        flying_star: Some(calc_flying_star(year)),
        period_info: selected.period_info.clone(),
        next_yun_start_year: next_yun,
        next_shi_start_year: next_shi,
        next_xun_start_year: next_xun,
        lunar: lunar_info,
        note: selected.note.clone(),
        mapping_record: selected.mapping_record.clone(),
        calc_meta: Some(FortuneCalcMeta {
            mode,
            primary: resolved_primary,
            year_start: year_start_label(year_start).to_string(),
            hj_year: year,
        }),
        variants: Some(FortuneVariants {
            algorithm: algorithm_variant,
            table_raw: table_raw_variant,
            table_normalized: table_normalized_variant,
        }),
        diff: Some(diff),
        authority: Some(authority),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;

    #[test]
    fn test_compute_fortune_default_compare_has_table_variant() {
        let req = FortuneRequest {
            datetime: Utc.with_ymd_and_hms(2026, 6, 1, 0, 0, 0).unwrap(),
            tz_offset_minutes: Some(480),
            lon: Some(116.4),
            use_true_solar_time: Some(false),
            mode: None,
            year_start: None,
            primary: None,
        };
        let resp = compute_fortune(&req);
        assert!(resp.calc_meta.is_some());
        assert!(resp.variants.is_some());
        let variants = resp.variants.expect("variants should exist");
        assert!(variants.table_raw.available, "2026 should hit table data");
        assert!(
            variants.table_normalized.available,
            "2026 should hit table data"
        );
    }

    #[test]
    fn test_compute_fortune_lichun_year_boundary() {
        let before = FortuneRequest {
            datetime: Utc.with_ymd_and_hms(2025, 2, 3, 12, 0, 0).unwrap(),
            tz_offset_minutes: Some(480),
            lon: Some(116.4),
            use_true_solar_time: Some(false),
            mode: Some(CalcMode::Algorithm),
            year_start: Some(YearStartMode::Lichun),
            primary: Some(PrimaryMode::Algorithm),
        };
        let after = FortuneRequest {
            datetime: Utc.with_ymd_and_hms(2025, 2, 5, 12, 0, 0).unwrap(),
            ..before.clone()
        };

        let before_resp = compute_fortune(&before);
        let after_resp = compute_fortune(&after);

        assert_eq!(
            before_resp.calc_meta.as_ref().map(|meta| meta.hj_year),
            Some(2024)
        );
        assert_eq!(
            after_resp.calc_meta.as_ref().map(|meta| meta.hj_year),
            Some(2025)
        );
    }

    #[test]
    fn test_requires_table_source() {
        assert!(requires_table_source(
            CalcMode::Table,
            PrimaryMode::Algorithm
        ));
        assert!(requires_table_source(CalcMode::Compare, PrimaryMode::Table));
        assert!(!requires_table_source(
            CalcMode::Compare,
            PrimaryMode::Algorithm
        ));
    }

    #[test]
    fn test_table_mode_in_coverage_stays_table() {
        let req = FortuneRequest {
            datetime: Utc.with_ymd_and_hms(2026, 6, 1, 0, 0, 0).unwrap(),
            tz_offset_minutes: Some(480),
            lon: Some(116.4),
            use_true_solar_time: Some(false),
            mode: Some(CalcMode::Table),
            year_start: Some(YearStartMode::GregorianNewYear),
            primary: Some(PrimaryMode::Table),
        };

        let resp = compute_fortune(&req);
        let meta = resp.calc_meta.expect("calc_meta should exist");
        let authority = resp.authority.expect("authority should exist");

        assert_eq!(meta.primary, PrimaryMode::Table);
        assert_eq!(authority.requested_source, "table");
        assert_eq!(authority.resolved_source, "table");
        assert_eq!(authority.fallback_reason, None);
        assert_eq!(authority.authority_level, "canonical");
        assert_eq!(
            authority
                .table_coverage
                .as_ref()
                .map(|coverage| coverage.covered),
            Some(true)
        );
    }

    #[test]
    fn test_table_mode_out_of_coverage_falls_back_to_algorithm() {
        let req = FortuneRequest {
            datetime: Utc.with_ymd_and_hms(1600, 6, 1, 0, 0, 0).unwrap(),
            tz_offset_minutes: Some(480),
            lon: Some(116.4),
            use_true_solar_time: Some(false),
            mode: Some(CalcMode::Table),
            year_start: Some(YearStartMode::GregorianNewYear),
            primary: Some(PrimaryMode::Table),
        };

        let resp = compute_fortune(&req);
        let variants = resp.variants.clone().expect("variants should exist");
        let meta = resp.calc_meta.expect("calc_meta should exist");
        let authority = resp.authority.expect("authority should exist");

        assert!(!variants.table_normalized.available);
        assert_eq!(meta.primary, PrimaryMode::Algorithm);
        assert_eq!(authority.requested_source, "table");
        assert_eq!(authority.resolved_source, "algorithm");
        assert_eq!(
            authority.fallback_reason,
            Some("table_not_covered".to_string())
        );
        assert_eq!(authority.authority_level, "derived");
        assert_eq!(
            authority
                .table_coverage
                .as_ref()
                .map(|coverage| coverage.covered),
            Some(false)
        );
    }
}
