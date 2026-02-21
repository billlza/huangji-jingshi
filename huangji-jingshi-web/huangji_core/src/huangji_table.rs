use crate::algorithm::FUXI_SEQ;
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct YearRecord {
    pub gregorian_year: i32,
    pub ganzhi: String,
    pub nian_hexagram: String,
    pub dynasty: String,
    pub person: String,
    pub yuan_raw: String,
    pub hui_raw: String,
    pub yun_raw: String,
    pub shi_raw: String,
    pub xun_raw: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NormalizedYearRecord {
    pub gregorian_year: i32,
    pub ganzhi: String,
    pub nian_hexagram_raw: String,
    pub nian_hexagram: Option<String>,
    pub dynasty: String,
    pub person: String,
    pub yuan_raw: String,
    pub hui_raw: String,
    pub yun_raw: String,
    pub shi_raw: String,
    pub xun_raw: String,
    pub hui_name: Option<String>,
    pub yun_name: Option<String>,
    pub shi_name: Option<String>,
    pub xun_name: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TableCoverage {
    pub min_year: i32,
    pub max_year: i32,
}

static YEAR_RECORDS: Lazy<Vec<YearRecord>> = Lazy::new(|| {
    serde_json::from_str(include_str!("../data/year_mapping.json")).unwrap_or_default()
});

static YEAR_RECORD_MAP: Lazy<HashMap<i32, YearRecord>> = Lazy::new(|| {
    YEAR_RECORDS
        .iter()
        .cloned()
        .map(|record| (record.gregorian_year, record))
        .collect()
});

static CANONICAL_HEXAGRAMS: Lazy<Vec<&'static str>> = Lazy::new(|| {
    let mut names: Vec<&'static str> = FUXI_SEQ.to_vec();
    names.sort_by_key(|name| usize::MAX - name.chars().count());
    names
});

static HEXAGRAM_ALIASES: Lazy<HashMap<&'static str, &'static str>> = Lazy::new(|| {
    HashMap::from([
        ("乾为天", "乾"),
        ("坤为地", "坤"),
        ("坎为水", "坎"),
        ("离为火", "离"),
        ("震为雷", "震"),
        ("巽为风", "巽"),
        ("兑为泽", "兑"),
        ("艮为山", "艮"),
        ("天地否", "否"),
        ("地天泰", "泰"),
        ("火雷噬阖", "噬嗑"),
        ("噬阖", "噬嗑"),
        ("风天小蓄", "小畜"),
        ("小蓄", "小畜"),
        ("天山屯", "屯"),
    ])
});

fn normalize_text(input: &str) -> String {
    input
        .trim()
        .replace('　', "")
        .replace('（', "(")
        .replace('）', ")")
        .replace('：', ":")
        .replace('濟', "济")
        .replace('澤', "泽")
        .replace('風', "风")
        .replace('蓄', "畜")
        .replace('闔', "嗑")
}

fn strip_range_suffix(input: &str) -> &str {
    let trimmed = input.trim();
    if let Some((head, _)) = trimmed.split_once('(') {
        head.trim()
    } else {
        trimmed
    }
}

fn extract_last_han(input: &str) -> Option<String> {
    input
        .chars()
        .rev()
        .find(|ch| ('\u{4E00}'..='\u{9FFF}').contains(ch))
        .map(|ch| ch.to_string())
}

fn contains_han(input: &str) -> bool {
    input
        .chars()
        .any(|ch| ('\u{4E00}'..='\u{9FFF}').contains(&ch))
}

pub fn normalize_hexagram_name(input: &str) -> Option<String> {
    let mut text = normalize_text(input);
    text = strip_range_suffix(&text).replace("卦", "");
    let text = text.trim();
    if text.is_empty() {
        return None;
    }

    if let Some(name) = HEXAGRAM_ALIASES.get(text) {
        return Some((*name).to_string());
    }
    if CANONICAL_HEXAGRAMS.contains(&text) {
        return Some(text.to_string());
    }
    for canonical in CANONICAL_HEXAGRAMS.iter() {
        if text.contains(canonical) {
            return Some((*canonical).to_string());
        }
    }
    None
}

pub fn normalize_year_record(record: &YearRecord) -> NormalizedYearRecord {
    let hui_name = if let Some((_, tail)) = record.hui_raw.split_once('会') {
        extract_last_han(tail)
    } else {
        None
    };
    let yun_name = if record.yun_raw.is_empty() {
        None
    } else if let Some(stripped) = record.yun_raw.trim().strip_suffix('运') {
        if contains_han(stripped) {
            Some(stripped.trim().to_string())
        } else {
            normalize_hexagram_name(stripped)
        }
    } else {
        normalize_hexagram_name(&record.yun_raw).or_else(|| extract_last_han(&record.yun_raw))
    };
    let shi_name = if let Some(stripped) = record.shi_raw.trim().strip_suffix('世') {
        if contains_han(stripped) {
            Some(stripped.trim().to_string())
        } else {
            None
        }
    } else if !record.shi_raw.trim().is_empty() {
        normalize_hexagram_name(&record.shi_raw).or_else(|| extract_last_han(&record.shi_raw))
    } else {
        None
    };
    let xun_name = normalize_hexagram_name(&record.xun_raw);

    NormalizedYearRecord {
        gregorian_year: record.gregorian_year,
        ganzhi: record.ganzhi.clone(),
        nian_hexagram_raw: record.nian_hexagram.clone(),
        nian_hexagram: normalize_hexagram_name(&record.nian_hexagram),
        dynasty: record.dynasty.clone(),
        person: record.person.clone(),
        yuan_raw: record.yuan_raw.clone(),
        hui_raw: record.hui_raw.clone(),
        yun_raw: record.yun_raw.clone(),
        shi_raw: record.shi_raw.clone(),
        xun_raw: record.xun_raw.clone(),
        hui_name,
        yun_name,
        shi_name,
        xun_name,
    }
}

pub fn get_year_record(year: i32) -> Option<YearRecord> {
    YEAR_RECORD_MAP.get(&year).cloned()
}

pub fn get_year_record_normalized(year: i32) -> Option<NormalizedYearRecord> {
    get_year_record(year).map(|record| normalize_year_record(&record))
}

pub fn get_coverage() -> Option<TableCoverage> {
    let min_year = YEAR_RECORDS.iter().map(|record| record.gregorian_year).min()?;
    let max_year = YEAR_RECORDS.iter().map(|record| record.gregorian_year).max()?;
    Some(TableCoverage { min_year, max_year })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normalize_hexagram_name() {
        assert_eq!(normalize_hexagram_name("天火同人"), Some("同人".to_string()));
        assert_eq!(normalize_hexagram_name("火雷噬阖"), Some("噬嗑".to_string()));
        assert_eq!(normalize_hexagram_name("风天小蓄"), Some("小畜".to_string()));
        assert_eq!(normalize_hexagram_name("天风姤（2024-2033）"), Some("姤".to_string()));
        assert_eq!(normalize_hexagram_name("坤为地"), Some("坤".to_string()));
    }

    #[test]
    fn test_get_year_record_2026() {
        let record = get_year_record(2026);
        assert!(record.is_some(), "2026 should exist in year_mapping.json");
        let normalized = get_year_record_normalized(2026).expect("normalized record");
        assert_eq!(normalized.nian_hexagram, Some("同人".to_string()));
    }
}
