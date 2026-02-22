# Data Contracts

This directory keeps two generations of year-level mapping files.

## `year_mapping.json` (legacy)

- Origin: early Excel extraction.
- Kept for backward compatibility and debugging (`table_raw` view).
- Known issues:
  - Field names do not match actual semantics.
  - `shi_raw` is empty for all rows.
  - Some rows place period information into `yuan_raw`.

## `year_mapping_canonical.json` (primary for table mode)

- Built by `cargo run -p huangji_core --bin generate_canonical_mapping`.
- Used by `huangji_core::table_engine` as the authoritative runtime source.
- Fields:
  - `year_hexagram`: canonical annual hexagram name.
  - `yuan_name/hui_name/yun_name/shi_name/xun_name`: canonical names for each level.
  - `*_index`: 1-based indices in the current hierarchy cycle.
  - `*_start_year/*_end_year`: inclusive range for each level.

## Runtime authority levels

- `canonical`
  - Source: `year_mapping_canonical.json`.
  - Meaning: direct table value inside canonical coverage (`1744-2103`).
  - Use when request source is table and year is covered.
- `derived`
  - Source: `algorithm` module (rule-based derivation aligned with古籍层级约束).
  - Meaning: traceable推导值，不伪造 table 原值。
  - Use when request source is algorithm, or request source is table but year is outside canonical coverage.

When fallback happens, API responses include:

- `requested_source` / `resolved_source`
- `table_coverage`
- `fallback_reason`
- `evidence_refs`

## Validation

Run:

```bash
cargo run -p huangji_core --bin validate_canonical_mapping
```

The validator enforces continuity, non-empty core fields, index bounds, and level-range consistency.
