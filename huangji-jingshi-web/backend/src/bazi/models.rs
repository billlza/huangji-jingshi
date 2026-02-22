use chrono::{DateTime, Datelike, FixedOffset, NaiveDateTime, Offset, TimeZone, Utc};
use chrono_tz::Tz;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

pub type ApiError = (axum::http::StatusCode, axum::Json<Value>);

#[derive(Debug, Deserialize)]
pub struct BaziQuery {
    pub datetime: String,
    pub timezone: Option<String>,
    #[serde(rename = "tzOffsetMinutes")]
    pub tz_offset_minutes: Option<i32>,
    #[allow(dead_code)]
    pub lat: Option<f64>,
    pub lon: Option<f64>,
    pub gender: Option<String>,
    pub source: Option<String>,
    #[serde(rename = "timeBasis")]
    pub time_basis: Option<String>,
    #[serde(rename = "dayRollover")]
    pub day_rollover: Option<String>,
    #[serde(rename = "useTrueSolarTime")]
    pub use_true_solar_time: Option<bool>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BaziSource {
    Auto,
    Sxtwl,
    HuangjiCore,
}

impl BaziSource {
    pub fn parse(raw: Option<&str>) -> Self {
        match raw.unwrap_or("auto").trim().to_ascii_lowercase().as_str() {
            "sxtwl" => Self::Sxtwl,
            "huangji_core" | "huangjicore" | "legacy" => Self::HuangjiCore,
            _ => Self::Auto,
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Auto => "auto",
            Self::Sxtwl => "sxtwl",
            Self::HuangjiCore => "huangji_core",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TimeBasis {
    Standard,
    TrueSolar,
}

impl TimeBasis {
    pub fn parse(raw: Option<&str>, use_true_solar_time: Option<bool>) -> Self {
        if matches!(use_true_solar_time, Some(true)) {
            return Self::TrueSolar;
        }

        match raw
            .unwrap_or("standard")
            .trim()
            .to_ascii_lowercase()
            .as_str()
        {
            "true_solar" | "truesolar" => Self::TrueSolar,
            _ => Self::Standard,
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Standard => "standard",
            Self::TrueSolar => "true_solar",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DayRollover {
    ZiChu23,
    ZiZheng00,
}

impl DayRollover {
    pub fn parse(raw: Option<&str>) -> Self {
        match raw
            .unwrap_or("zi_chu_23")
            .trim()
            .to_ascii_lowercase()
            .as_str()
        {
            "zi_zheng_00" | "zizheng00" | "00:00" => Self::ZiZheng00,
            _ => Self::ZiChu23,
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::ZiChu23 => "zi_chu_23",
            Self::ZiZheng00 => "zi_zheng_00",
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct RuleProfile {
    pub time_basis: String,
    pub day_rollover: String,
    pub timezone: Option<String>,
    pub tz_offset_minutes: i32,
    pub longitude: f64,
}

#[derive(Debug, Clone, Serialize)]
pub struct AuthorityEvidenceRef {
    pub label: String,
    pub url_or_id: String,
    pub version: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct AuthorityMeta {
    pub requested_source: String,
    pub resolved_source: String,
    pub fallback_reason: Option<String>,
    pub authority_level: String,
    pub rule_profile: RuleProfile,
    pub evidence_refs: Vec<AuthorityEvidenceRef>,
}

#[derive(Debug, Clone, Serialize)]
pub struct BaziVariant {
    pub available: bool,
    pub reason: Option<String>,
    pub payload: Option<Value>,
}

impl BaziVariant {
    pub fn unavailable(reason: impl Into<String>) -> Self {
        Self {
            available: false,
            reason: Some(reason.into()),
            payload: None,
        }
    }

    pub fn available(payload: Value) -> Self {
        Self {
            available: true,
            reason: None,
            payload: Some(payload),
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct BaziVariants {
    pub sxtwl: BaziVariant,
    pub huangji_core: BaziVariant,
}

#[derive(Debug, Clone)]
pub struct PillarIndices {
    pub year: (usize, usize),
    pub month: (usize, usize),
    pub day: (usize, usize),
    pub hour: (usize, usize),
    pub solar_longitude: Option<f64>,
    pub solar_term: Option<String>,
    pub true_solar_hour: Option<f64>,
    pub is_late_zi: bool,
}

#[derive(Debug, Clone)]
pub struct EngineResult {
    pub source: BaziSource,
    pub payload: Value,
    pub authority_level: &'static str,
    pub evidence_refs: Vec<AuthorityEvidenceRef>,
}

#[derive(Debug, Clone)]
pub enum ZoneContext {
    Iana { name: String, tz: Tz },
    Fixed(FixedOffset),
}

impl ZoneContext {
    pub fn timezone_name(&self) -> Option<String> {
        match self {
            Self::Iana { name, .. } => Some(name.clone()),
            Self::Fixed(_) => None,
        }
    }

    pub fn offset_minutes_at_utc(&self, dt: DateTime<Utc>) -> i32 {
        match self {
            Self::Iana { tz, .. } => dt.with_timezone(tz).offset().fix().local_minus_utc() / 60,
            Self::Fixed(offset) => offset.local_minus_utc() / 60,
        }
    }

    pub fn birth_year(&self, dt: DateTime<Utc>) -> i32 {
        match self {
            Self::Iana { tz, .. } => dt.with_timezone(tz).year(),
            Self::Fixed(offset) => dt.with_timezone(offset).year(),
        }
    }

    pub fn current_year(&self) -> i32 {
        match self {
            Self::Iana { tz, .. } => Utc::now().with_timezone(tz).year(),
            Self::Fixed(offset) => Utc::now().with_timezone(offset).year(),
        }
    }

    pub fn local_datetime(&self, dt: DateTime<Utc>) -> NaiveDateTime {
        match self {
            Self::Iana { tz, .. } => dt.with_timezone(tz).naive_local(),
            Self::Fixed(offset) => dt.with_timezone(offset).naive_local(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct BaziRequestContext {
    pub datetime_utc: DateTime<Utc>,
    pub zone: ZoneContext,
    pub tz_offset_minutes: i32,
    pub longitude: f64,
    pub gender: String,
    pub requested_source: BaziSource,
    pub time_basis: TimeBasis,
    pub day_rollover: DayRollover,
}

impl BaziRequestContext {
    pub fn rule_profile(&self) -> RuleProfile {
        RuleProfile {
            time_basis: self.time_basis.as_str().to_string(),
            day_rollover: self.day_rollover.as_str().to_string(),
            timezone: self.zone.timezone_name(),
            tz_offset_minutes: self.tz_offset_minutes,
            longitude: self.longitude,
        }
    }

    pub fn birth_year(&self) -> i32 {
        self.zone.birth_year(self.datetime_utc)
    }

    pub fn current_year(&self) -> i32 {
        self.zone.current_year()
    }
}

fn invalid_request(error: &str, message: impl Into<String>) -> ApiError {
    (
        axum::http::StatusCode::BAD_REQUEST,
        axum::Json(json!({
            "error": error,
            "message": message.into(),
        })),
    )
}

fn parse_timezone(name: &str) -> Result<Tz, ApiError> {
    name.parse::<Tz>().map_err(|_| {
        invalid_request(
            "invalid_timezone",
            format!(
                "invalid timezone '{}', expected an IANA timezone like Asia/Shanghai",
                name
            ),
        )
    })
}

pub fn parse_datetime_with_zone(
    raw: &str,
    timezone: Option<&str>,
    tz_offset_minutes: Option<i32>,
) -> Result<(DateTime<Utc>, ZoneContext), ApiError> {
    let timezone_name = timezone.map(str::trim).filter(|value| !value.is_empty());

    if let Ok(dt) = DateTime::parse_from_rfc3339(raw) {
        let utc = dt.with_timezone(&Utc);
        if let Some(name) = timezone_name {
            let tz = parse_timezone(name)?;
            return Ok((
                utc,
                ZoneContext::Iana {
                    name: name.to_string(),
                    tz,
                },
            ));
        }

        let parsed_offset = dt.offset().fix().local_minus_utc() / 60;
        let offset_minutes = tz_offset_minutes.unwrap_or(parsed_offset);
        let fixed = FixedOffset::east_opt(offset_minutes * 60).ok_or_else(|| {
            invalid_request(
                "invalid_tz_offset",
                format!(
                    "invalid tzOffsetMinutes '{}' for datetime '{}'",
                    offset_minutes, raw
                ),
            )
        })?;
        return Ok((utc, ZoneContext::Fixed(fixed)));
    }

    let naive = NaiveDateTime::parse_from_str(raw, "%Y-%m-%dT%H:%M:%S")
        .or_else(|_| NaiveDateTime::parse_from_str(raw, "%Y-%m-%dT%H:%M"))
        .map_err(|_| {
            invalid_request(
                "invalid_datetime",
                format!(
                    "invalid datetime '{}', expected RFC3339 or local ISO format (YYYY-MM-DDTHH:MM[:SS])",
                    raw
                ),
            )
        })?;

    if let Some(name) = timezone_name {
        let tz = parse_timezone(name)?;
        let local = tz
            .from_local_datetime(&naive)
            .single()
            .or_else(|| tz.from_local_datetime(&naive).earliest())
            .or_else(|| tz.from_local_datetime(&naive).latest())
            .ok_or_else(|| {
                invalid_request(
                    "invalid_local_datetime",
                    format!(
                        "cannot resolve local datetime '{}' in timezone '{}'",
                        raw, name
                    ),
                )
            })?;
        return Ok((
            local.with_timezone(&Utc),
            ZoneContext::Iana {
                name: name.to_string(),
                tz,
            },
        ));
    }

    let offset_minutes = tz_offset_minutes.unwrap_or(480);
    let fixed = FixedOffset::east_opt(offset_minutes * 60).ok_or_else(|| {
        invalid_request(
            "invalid_tz_offset",
            format!(
                "invalid tzOffsetMinutes '{}' for datetime '{}'",
                offset_minutes, raw
            ),
        )
    })?;
    let local = fixed.from_local_datetime(&naive).single().ok_or_else(|| {
        invalid_request(
            "invalid_local_datetime",
            format!(
                "cannot resolve local datetime '{}' with tzOffsetMinutes '{}'",
                raw, offset_minutes
            ),
        )
    })?;
    Ok((local.with_timezone(&Utc), ZoneContext::Fixed(fixed)))
}
