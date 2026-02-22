use huangji_core::table_engine;

fn ensure(condition: bool, message: &str) -> anyhow::Result<()> {
    if condition {
        Ok(())
    } else {
        Err(anyhow::anyhow!(message.to_string()))
    }
}

fn main() -> anyhow::Result<()> {
    let records = table_engine::get_all_records();
    ensure(!records.is_empty(), "canonical mapping is empty")?;

    let coverage = table_engine::get_coverage()
        .ok_or_else(|| anyhow::anyhow!("canonical coverage missing"))?;

    for pair in records.windows(2) {
        let prev = &pair[0];
        let next = &pair[1];
        ensure(
            next.gregorian_year == prev.gregorian_year + 1,
            &format!(
                "year continuity broken: {} -> {}",
                prev.gregorian_year, next.gregorian_year
            ),
        )?;
    }

    for record in &records {
        ensure(
            !record.yuan_name.trim().is_empty(),
            &format!("empty yuan_name: {}", record.gregorian_year),
        )?;
        ensure(
            !record.hui_name.trim().is_empty(),
            &format!("empty hui_name: {}", record.gregorian_year),
        )?;
        ensure(
            !record.yun_name.trim().is_empty(),
            &format!("empty yun_name: {}", record.gregorian_year),
        )?;
        ensure(
            !record.shi_name.trim().is_empty(),
            &format!("empty shi_name: {}", record.gregorian_year),
        )?;
        ensure(
            !record.xun_name.trim().is_empty(),
            &format!("empty xun_name: {}", record.gregorian_year),
        )?;

        ensure(
            (1..=12).contains(&record.hui_index),
            &format!("invalid hui_index: {}", record.gregorian_year),
        )?;
        ensure(
            (1..=30).contains(&record.yun_index),
            &format!("invalid yun_index: {}", record.gregorian_year),
        )?;
        ensure(
            (1..=12).contains(&record.shi_index),
            &format!("invalid shi_index: {}", record.gregorian_year),
        )?;
        ensure(
            (1..=3).contains(&record.xun_index),
            &format!("invalid xun_index: {}", record.gregorian_year),
        )?;

        ensure(
            record.gregorian_year >= record.yuan_start_year
                && record.gregorian_year <= record.yuan_end_year,
            &format!("year out of yuan range: {}", record.gregorian_year),
        )?;
        ensure(
            record.gregorian_year >= record.hui_start_year
                && record.gregorian_year <= record.hui_end_year,
            &format!("year out of hui range: {}", record.gregorian_year),
        )?;
        ensure(
            record.gregorian_year >= record.yun_start_year
                && record.gregorian_year <= record.yun_end_year,
            &format!("year out of yun range: {}", record.gregorian_year),
        )?;
        ensure(
            record.gregorian_year >= record.shi_start_year
                && record.gregorian_year <= record.shi_end_year,
            &format!("year out of shi range: {}", record.gregorian_year),
        )?;
        ensure(
            record.gregorian_year >= record.xun_start_year
                && record.gregorian_year <= record.xun_end_year,
            &format!("year out of xun range: {}", record.gregorian_year),
        )?;

        ensure(
            record.yun_end_year - record.yun_start_year + 1 == 360,
            &format!("yun length mismatch: {}", record.gregorian_year),
        )?;
        ensure(
            record.shi_end_year - record.shi_start_year + 1 == 30,
            &format!("shi length mismatch: {}", record.gregorian_year),
        )?;
        ensure(
            record.xun_end_year - record.xun_start_year + 1 == 10,
            &format!("xun length mismatch: {}", record.gregorian_year),
        )?;
    }

    println!(
        "Canonical mapping validation passed: {} rows ({}-{})",
        records.len(),
        coverage.min_year,
        coverage.max_year
    );
    Ok(())
}
