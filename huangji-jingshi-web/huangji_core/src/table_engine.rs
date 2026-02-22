use crate::algorithm::{self, HuangjiInfo, PeriodInfo, TimelineData};
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CanonicalYearRecord {
    pub gregorian_year: i32,
    pub ganzhi: String,
    pub year_hexagram: String,
    pub yuan_name: String,
    pub hui_name: String,
    pub yun_name: String,
    pub shi_name: String,
    pub xun_name: String,
    pub yuan_index: u32,
    pub hui_index: u32,
    pub yun_index: u32,
    pub shi_index: u32,
    pub xun_index: u32,
    pub yuan_start_year: i32,
    pub yuan_end_year: i32,
    pub hui_start_year: i32,
    pub hui_end_year: i32,
    pub yun_start_year: i32,
    pub yun_end_year: i32,
    pub shi_start_year: i32,
    pub shi_end_year: i32,
    pub xun_start_year: i32,
    pub xun_end_year: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TableCoverage {
    pub min_year: i32,
    pub max_year: i32,
}

#[derive(Debug, Clone, Copy)]
enum TimelineLevel {
    Yuan,
    Hui,
    Yun,
    Shi,
    Xun,
}

static CANONICAL_RECORDS: Lazy<Vec<CanonicalYearRecord>> = Lazy::new(|| {
    serde_json::from_str(include_str!("../data/year_mapping_canonical.json")).unwrap_or_default()
});

static CANONICAL_MAP: Lazy<HashMap<i32, CanonicalYearRecord>> = Lazy::new(|| {
    CANONICAL_RECORDS
        .iter()
        .cloned()
        .map(|record| (record.gregorian_year, record))
        .collect()
});

pub fn get_all_records() -> Vec<CanonicalYearRecord> {
    CANONICAL_RECORDS.clone()
}

pub fn get_coverage() -> Option<TableCoverage> {
    let min_year = CANONICAL_RECORDS
        .iter()
        .map(|record| record.gregorian_year)
        .min()?;
    let max_year = CANONICAL_RECORDS
        .iter()
        .map(|record| record.gregorian_year)
        .max()?;
    Some(TableCoverage { min_year, max_year })
}

pub fn has_year(year: i32) -> bool {
    CANONICAL_MAP.contains_key(&year)
}

pub fn get_year_record(year: i32) -> Option<CanonicalYearRecord> {
    CANONICAL_MAP.get(&year).cloned()
}

fn period_info(
    name: String,
    start_year: i32,
    end_year: i32,
    index: u32,
    max_index: u32,
) -> PeriodInfo {
    PeriodInfo {
        name,
        start_year,
        end_year,
        index,
        max_index,
    }
}

fn level_name(record: &CanonicalYearRecord, level: TimelineLevel) -> String {
    match level {
        TimelineLevel::Yuan => record.yuan_name.clone(),
        TimelineLevel::Hui => record.hui_name.clone(),
        TimelineLevel::Yun => record.yun_name.clone(),
        TimelineLevel::Shi => record.shi_name.clone(),
        TimelineLevel::Xun => record.xun_name.clone(),
    }
}

fn level_index(record: &CanonicalYearRecord, level: TimelineLevel) -> u32 {
    match level {
        TimelineLevel::Yuan => record.yuan_index,
        TimelineLevel::Hui => record.hui_index,
        TimelineLevel::Yun => record.yun_index,
        TimelineLevel::Shi => record.shi_index,
        TimelineLevel::Xun => record.xun_index,
    }
}

fn period_name_from_canonical(
    level: TimelineLevel,
    start_year: i32,
    end_year: i32,
    expected_index: u32,
) -> Option<String> {
    let coverage = get_coverage()?;
    let start = start_year.max(coverage.min_year);
    let end = end_year.min(coverage.max_year);
    if start > end {
        return None;
    }

    for year in start..=end {
        if let Some(record) = CANONICAL_MAP.get(&year) {
            if level_index(record, level) == expected_index {
                return Some(level_name(record, level));
            }
        }
    }

    for year in start..=end {
        if let Some(record) = CANONICAL_MAP.get(&year) {
            return Some(level_name(record, level));
        }
    }

    None
}

fn assign_period_name(level: TimelineLevel, period: &PeriodInfo) -> String {
    period_name_from_canonical(level, period.start_year, period.end_year, period.index)
        .unwrap_or_else(|| "未载".to_string())
}

fn apply_names(level: TimelineLevel, list: &mut [PeriodInfo], current: &PeriodInfo) {
    for item in list.iter_mut() {
        item.name = assign_period_name(level, item);
    }

    if let Some(item) = list
        .iter_mut()
        .find(|item| item.start_year == current.start_year && item.end_year == current.end_year)
    {
        item.name = current.name.clone();
        return;
    }

    if let Some(item) = list.iter_mut().find(|item| item.index == current.index) {
        item.name = current.name.clone();
    }
}

pub fn get_hj_info(year: i32) -> Option<HuangjiInfo> {
    let record = get_year_record(year)?;
    Some(HuangjiInfo {
        yuan: period_info(
            record.yuan_name.clone(),
            record.yuan_start_year,
            record.yuan_end_year,
            record.yuan_index,
            1,
        ),
        hui: period_info(
            record.hui_name.clone(),
            record.hui_start_year,
            record.hui_end_year,
            record.hui_index,
            12,
        ),
        yun: period_info(
            record.yun_name.clone(),
            record.yun_start_year,
            record.yun_end_year,
            record.yun_index,
            30,
        ),
        shi: period_info(
            record.shi_name.clone(),
            record.shi_start_year,
            record.shi_end_year,
            record.shi_index,
            12,
        ),
        xun: period_info(
            record.xun_name.clone(),
            record.xun_start_year,
            record.xun_end_year,
            record.xun_index,
            3,
        ),
        year_gua: record.year_hexagram,
    })
}

pub fn get_timeline_info(year: i32) -> Option<TimelineData> {
    let current = get_hj_info(year)?;
    let mut timeline = algorithm::get_timeline_info(year);
    timeline.current = current.clone();

    apply_names(TimelineLevel::Yuan, &mut timeline.yuan_list, &current.yuan);
    apply_names(TimelineLevel::Hui, &mut timeline.hui_list, &current.hui);
    apply_names(TimelineLevel::Yun, &mut timeline.yun_list, &current.yun);
    apply_names(TimelineLevel::Shi, &mut timeline.shi_list, &current.shi);
    apply_names(TimelineLevel::Xun, &mut timeline.xun_list, &current.xun);

    Some(timeline)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_coverage_exists() {
        let coverage = get_coverage().expect("canonical coverage must exist");
        assert!(coverage.min_year <= 1744);
        assert!(coverage.max_year >= 2103);
    }

    #[test]
    fn test_record_2026_non_empty_fields() {
        let record = get_year_record(2026).expect("canonical record 2026");
        assert!(!record.yun_name.trim().is_empty());
        assert!(!record.shi_name.trim().is_empty());
        assert!(!record.xun_name.trim().is_empty());
    }

    #[test]
    fn test_timeline_current_matches_list_for_2026() {
        let timeline = get_timeline_info(2026).expect("canonical timeline 2026");
        let current_xun = timeline.current.xun.name.clone();
        let list_item = timeline
            .xun_list
            .iter()
            .find(|item| item.index == timeline.current.xun.index)
            .expect("xun list item");
        assert_eq!(list_item.name, current_xun);
    }
}
