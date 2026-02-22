use huangji_core::{algorithm, huangji_table};
use serde::Serialize;
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Serialize)]
struct CanonicalYearRecord {
    gregorian_year: i32,
    ganzhi: String,
    year_hexagram: String,
    yuan_name: String,
    hui_name: String,
    yun_name: String,
    shi_name: String,
    xun_name: String,
    yuan_index: u32,
    hui_index: u32,
    yun_index: u32,
    shi_index: u32,
    xun_index: u32,
    yuan_start_year: i32,
    yuan_end_year: i32,
    hui_start_year: i32,
    hui_end_year: i32,
    yun_start_year: i32,
    yun_end_year: i32,
    shi_start_year: i32,
    shi_end_year: i32,
    xun_start_year: i32,
    xun_end_year: i32,
}

fn main() -> anyhow::Result<()> {
    let coverage =
        huangji_table::get_coverage().ok_or_else(|| anyhow::anyhow!("legacy coverage missing"))?;
    let mut records = Vec::new();

    for year in coverage.min_year..=coverage.max_year {
        let algo_info = algorithm::get_hj_info(year);
        let normalized = huangji_table::get_year_record_normalized(year)
            .ok_or_else(|| anyhow::anyhow!("missing legacy normalized record for {}", year))?;

        let year_hexagram = normalized
            .nian_hexagram
            .unwrap_or_else(|| algo_info.year_gua.clone());
        let yuan_name = huangji_table::normalize_hexagram_name(&normalized.yuan_raw)
            .unwrap_or_else(|| algo_info.yuan.name.clone());
        let hui_name = normalized
            .hui_name
            .unwrap_or_else(|| algo_info.hui.name.clone());
        let yun_name = normalized
            .yun_name
            .unwrap_or_else(|| algo_info.yun.name.clone());
        let shi_name = normalized
            .shi_name
            .unwrap_or_else(|| algo_info.shi.name.clone());
        let xun_name = normalized
            .xun_name
            .unwrap_or_else(|| algo_info.xun.name.clone());

        records.push(CanonicalYearRecord {
            gregorian_year: year,
            ganzhi: normalized.ganzhi,
            year_hexagram,
            yuan_name,
            hui_name,
            yun_name,
            shi_name,
            xun_name,
            yuan_index: algo_info.yuan.index,
            hui_index: algo_info.hui.index,
            yun_index: algo_info.yun.index,
            shi_index: algo_info.shi.index,
            xun_index: algo_info.xun.index,
            yuan_start_year: algo_info.yuan.start_year,
            yuan_end_year: algo_info.yuan.end_year,
            hui_start_year: algo_info.hui.start_year,
            hui_end_year: algo_info.hui.end_year,
            yun_start_year: algo_info.yun.start_year,
            yun_end_year: algo_info.yun.end_year,
            shi_start_year: algo_info.shi.start_year,
            shi_end_year: algo_info.shi.end_year,
            xun_start_year: algo_info.xun.start_year,
            xun_end_year: algo_info.xun.end_year,
        });
    }

    let output = serde_json::to_string_pretty(&records)?;
    let output_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("data")
        .join("year_mapping_canonical.json");
    fs::write(&output_path, output)?;

    println!(
        "Generated canonical mapping: {} records -> {}",
        records.len(),
        output_path.display()
    );
    Ok(())
}
