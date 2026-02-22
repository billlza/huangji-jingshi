use crate::bazi::models::{BaziRequestContext, PillarIndices};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::path::PathBuf;
use std::process::Stdio;
use tokio::io::AsyncWriteExt;
use tokio::process::Command;
use tokio::time::{timeout, Duration};

#[derive(Debug, Clone)]
pub struct BridgeFailure {
    pub reason: String,
    pub detail: String,
}

#[derive(Debug, Clone)]
pub struct BridgeSuccess {
    pub pillars: PillarIndices,
    pub engine_version: Option<String>,
}

#[derive(Debug, Serialize)]
struct ScriptInput {
    datetime_utc: String,
    timezone: Option<String>,
    tz_offset_minutes: i32,
    longitude: f64,
    time_basis: String,
    day_rollover: String,
}

#[derive(Debug, Deserialize, Serialize)]
struct ScriptGanzhi {
    tg: usize,
    dz: usize,
}

#[derive(Debug, Deserialize)]
struct ScriptOutput {
    ok: bool,
    error_code: Option<String>,
    message: Option<String>,
    engine_version: Option<String>,
    year: Option<ScriptGanzhi>,
    month: Option<ScriptGanzhi>,
    day: Option<ScriptGanzhi>,
    hour: Option<ScriptGanzhi>,
    solar_term: Option<String>,
    solar_longitude: Option<f64>,
    true_solar_hour: Option<f64>,
    is_late_zi: Option<bool>,
}

fn script_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("scripts")
        .join("bazi_sxtwl.py")
}

fn python_binary() -> String {
    std::env::var("BAZI_PYTHON_BIN").unwrap_or_else(|_| "python3".to_string())
}

fn sxtwl_timeout_ms() -> u64 {
    std::env::var("BAZI_SXTWL_TIMEOUT_MS")
        .ok()
        .and_then(|value| value.parse::<u64>().ok())
        .filter(|value| *value >= 200 && *value <= 10_000)
        .unwrap_or(2_000)
}

fn map_script_failure(code: Option<&str>, message: String) -> BridgeFailure {
    match code.unwrap_or("runtime_error") {
        "module_not_available" => BridgeFailure {
            reason: "primary_unavailable".to_string(),
            detail: message,
        },
        "out_of_supported_range" => BridgeFailure {
            reason: "primary_out_of_supported_range".to_string(),
            detail: message,
        },
        _ => BridgeFailure {
            reason: "primary_runtime_error".to_string(),
            detail: message,
        },
    }
}

fn validate_pair(pair: &ScriptGanzhi) -> bool {
    pair.tg < 10 && pair.dz < 12
}

pub fn log_sxtwl_health() {
    let script = script_path();
    if !script.exists() {
        tracing::warn!(
            "⚠️ sxtwl script missing, bazi primary source will fallback: {}",
            script.display()
        );
        return;
    }

    let output = std::process::Command::new(python_binary())
        .arg(&script)
        .arg("--health")
        .output();

    match output {
        Ok(result) if result.status.success() => {
            let stdout = String::from_utf8_lossy(&result.stdout);
            tracing::info!("✅ sxtwl health: {}", stdout.trim());
        }
        Ok(result) => {
            let stderr = String::from_utf8_lossy(&result.stderr);
            tracing::warn!(
                "⚠️ sxtwl health check failed (status={}): {}",
                result.status,
                stderr.trim()
            );
        }
        Err(error) => {
            tracing::warn!("⚠️ sxtwl health check spawn failed: {}", error);
        }
    }
}

pub async fn compute_with_sxtwl(ctx: &BaziRequestContext) -> Result<BridgeSuccess, BridgeFailure> {
    let script = script_path();
    if !script.exists() {
        return Err(BridgeFailure {
            reason: "primary_unavailable".to_string(),
            detail: format!("script not found: {}", script.display()),
        });
    }

    let payload = ScriptInput {
        datetime_utc: ctx.datetime_utc.to_rfc3339(),
        timezone: ctx.zone.timezone_name(),
        tz_offset_minutes: ctx.tz_offset_minutes,
        longitude: ctx.longitude,
        time_basis: ctx.time_basis.as_str().to_string(),
        day_rollover: ctx.day_rollover.as_str().to_string(),
    };

    let mut child = Command::new(python_binary())
        .arg(&script)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|error| BridgeFailure {
            reason: "primary_unavailable".to_string(),
            detail: format!("spawn failed: {}", error),
        })?;

    let input = serde_json::to_vec(&payload).map_err(|error| BridgeFailure {
        reason: "primary_runtime_error".to_string(),
        detail: format!("serialize input failed: {}", error),
    })?;

    if let Some(mut stdin) = child.stdin.take() {
        stdin
            .write_all(&input)
            .await
            .map_err(|error| BridgeFailure {
                reason: "primary_runtime_error".to_string(),
                detail: format!("write stdin failed: {}", error),
            })?;
    }

    let timeout_ms = sxtwl_timeout_ms();
    let output = timeout(Duration::from_millis(timeout_ms), child.wait_with_output())
        .await
        .map_err(|_| BridgeFailure {
            reason: "primary_timeout".to_string(),
            detail: format!("sxtwl execution exceeded {}ms timeout", timeout_ms),
        })?
        .map_err(|error| BridgeFailure {
            reason: "primary_runtime_error".to_string(),
            detail: format!("wait output failed: {}", error),
        })?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
        return Err(BridgeFailure {
            reason: "primary_runtime_error".to_string(),
            detail: format!("script exit status {}: {}", output.status, stderr),
        });
    }

    let parsed: ScriptOutput =
        serde_json::from_slice(&output.stdout).map_err(|error| BridgeFailure {
            reason: "primary_runtime_error".to_string(),
            detail: format!(
                "invalid script json: {} ({})",
                error,
                String::from_utf8_lossy(&output.stdout)
            ),
        })?;

    if !parsed.ok {
        return Err(map_script_failure(
            parsed.error_code.as_deref(),
            parsed
                .message
                .unwrap_or_else(|| "script returned not ok".to_string()),
        ));
    }

    let year = parsed.year.ok_or_else(|| BridgeFailure {
        reason: "primary_runtime_error".to_string(),
        detail: "missing year pillar".to_string(),
    })?;
    let month = parsed.month.ok_or_else(|| BridgeFailure {
        reason: "primary_runtime_error".to_string(),
        detail: "missing month pillar".to_string(),
    })?;
    let day = parsed.day.ok_or_else(|| BridgeFailure {
        reason: "primary_runtime_error".to_string(),
        detail: "missing day pillar".to_string(),
    })?;
    let hour = parsed.hour.ok_or_else(|| BridgeFailure {
        reason: "primary_runtime_error".to_string(),
        detail: "missing hour pillar".to_string(),
    })?;

    if !(validate_pair(&year)
        && validate_pair(&month)
        && validate_pair(&day)
        && validate_pair(&hour))
    {
        return Err(BridgeFailure {
            reason: "primary_runtime_error".to_string(),
            detail: format!(
                "invalid ganzhi indices: {}",
                json!({ "year": year, "month": month, "day": day, "hour": hour })
            ),
        });
    }

    Ok(BridgeSuccess {
        pillars: PillarIndices {
            year: (year.tg, year.dz),
            month: (month.tg, month.dz),
            day: (day.tg, day.dz),
            hour: (hour.tg, hour.dz),
            solar_longitude: parsed.solar_longitude,
            solar_term: parsed.solar_term,
            true_solar_hour: parsed.true_solar_hour,
            is_late_zi: parsed.is_late_zi.unwrap_or(false),
        },
        engine_version: parsed.engine_version,
    })
}
