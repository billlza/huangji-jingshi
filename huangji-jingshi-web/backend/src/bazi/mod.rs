mod legacy;
mod models;
mod resolver;
mod sxtwl_bridge;

use axum::{extract::Query, http::StatusCode, Json};
use models::{
    parse_datetime_with_zone, ApiError, BaziQuery, BaziRequestContext, BaziSource, DayRollover,
    TimeBasis,
};
use resolver::resolve_bazi;

pub async fn get_bazi(
    Query(params): Query<BaziQuery>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    tracing::info!(
        "🔮 八字排盘请求: datetime={}, source={:?}, timeBasis={:?}, dayRollover={:?}, lon={:?}",
        params.datetime,
        params.source,
        params.time_basis,
        params.day_rollover,
        params.lon
    );

    let context = parse_request_context(params)?;
    let response = resolve_bazi(&context).await?;
    Ok(Json(response))
}

pub fn log_sxtwl_health() {
    sxtwl_bridge::log_sxtwl_health();
}

fn parse_request_context(params: BaziQuery) -> Result<BaziRequestContext, ApiError> {
    let requested_source = BaziSource::parse(params.source.as_deref());
    let time_basis = TimeBasis::parse(params.time_basis.as_deref(), params.use_true_solar_time);
    let day_rollover = DayRollover::parse(params.day_rollover.as_deref());

    let timezone = params.timezone.as_deref();
    let (datetime_utc, zone) =
        parse_datetime_with_zone(&params.datetime, timezone, params.tz_offset_minutes)?;
    let tz_offset_minutes = zone.offset_minutes_at_utc(datetime_utc);

    let gender = params
        .gender
        .unwrap_or_else(|| "male".to_string())
        .trim()
        .to_ascii_lowercase();
    let gender = match gender.as_str() {
        "male" | "female" | "other" => gender,
        _ => "male".to_string(),
    };

    Ok(BaziRequestContext {
        datetime_utc,
        zone,
        tz_offset_minutes,
        longitude: params.lon.unwrap_or(116.4),
        gender,
        requested_source,
        time_basis,
        day_rollover,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use models::BaziQuery;
    use serde_json::json;

    #[tokio::test]
    async fn invalid_datetime_returns_400() {
        let query = BaziQuery {
            datetime: "bad-input".to_string(),
            timezone: Some("Asia/Shanghai".to_string()),
            tz_offset_minutes: Some(480),
            lat: None,
            lon: Some(116.4),
            gender: Some("male".to_string()),
            source: Some("auto".to_string()),
            time_basis: Some("standard".to_string()),
            day_rollover: Some("zi_chu_23".to_string()),
            use_true_solar_time: None,
        };

        let result = parse_request_context(query);
        assert!(result.is_err());
        let err = result.err().expect("should return bad request");
        assert_eq!(err.0, StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    async fn response_top_level_matches_resolved_variant() {
        let query = BaziQuery {
            datetime: "2002-07-19T23:15:00".to_string(),
            timezone: Some("Asia/Shanghai".to_string()),
            tz_offset_minutes: Some(480),
            lat: Some(39.08449),
            lon: Some(117.195672),
            gender: Some("male".to_string()),
            source: Some("auto".to_string()),
            time_basis: Some("standard".to_string()),
            day_rollover: Some("zi_chu_23".to_string()),
            use_true_solar_time: None,
        };

        let context = parse_request_context(query).expect("context");
        let response = resolve_bazi(&context).await.expect("resolved response");
        let authority = response.get("authority").expect("authority");
        let resolved = authority
            .get("resolved_source")
            .and_then(serde_json::Value::as_str)
            .expect("resolved source");
        let variants = response
            .get("variants")
            .expect("variants")
            .as_object()
            .expect("variants object");
        let variant = variants.get(resolved).expect("resolved variant");
        let variant_payload = variant
            .get("payload")
            .and_then(serde_json::Value::as_object)
            .expect("variant payload");

        for field in ["year_pillar", "month_pillar", "day_pillar", "hour_pillar"] {
            assert_eq!(
                response.get(field),
                variant_payload.get(field),
                "top-level field '{}' should match resolved variant payload",
                field
            );
        }
    }

    #[tokio::test]
    async fn source_sxtwl_reports_fallback_when_primary_not_available() {
        let query = BaziQuery {
            datetime: "2002-07-19T23:15:00".to_string(),
            timezone: Some("Asia/Shanghai".to_string()),
            tz_offset_minutes: Some(480),
            lat: Some(39.08449),
            lon: Some(117.195672),
            gender: Some("male".to_string()),
            source: Some("sxtwl".to_string()),
            time_basis: Some("standard".to_string()),
            day_rollover: Some("zi_chu_23".to_string()),
            use_true_solar_time: None,
        };
        let context = parse_request_context(query).expect("context");
        let response = resolve_bazi(&context).await.expect("resolved response");
        let authority = response
            .get("authority")
            .cloned()
            .unwrap_or_else(|| json!({}));
        let resolved_source = authority
            .get("resolved_source")
            .and_then(serde_json::Value::as_str)
            .expect("resolved source");

        if resolved_source == "huangji_core" {
            assert!(authority.get("fallback_reason").is_some());
        } else {
            assert_eq!(resolved_source, "sxtwl");
        }
    }
}
