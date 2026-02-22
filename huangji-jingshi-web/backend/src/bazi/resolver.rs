use crate::bazi::legacy;
use crate::bazi::models::{
    ApiError, AuthorityEvidenceRef, AuthorityMeta, BaziRequestContext, BaziSource, BaziVariant,
    BaziVariants, EngineResult,
};
use crate::bazi::sxtwl_bridge::{compute_with_sxtwl, BridgeFailure};
use axum::{http::StatusCode, Json};
use serde_json::{json, Value};

fn engine_evidence(engine: BaziSource, version: Option<String>) -> Vec<AuthorityEvidenceRef> {
    match engine {
        BaziSource::Sxtwl => vec![
            AuthorityEvidenceRef {
                label: "sxtwl (寿星天文历)".to_string(),
                url_or_id: "pypi:sxtwl".to_string(),
                version,
            },
            AuthorityEvidenceRef {
                label: "子平真诠（大运规则）".to_string(),
                url_or_id: "wikisource:子平真诠".to_string(),
                version: None,
            },
        ],
        BaziSource::HuangjiCore | BaziSource::Auto => vec![
            AuthorityEvidenceRef {
                label: "huangji_core::calendar::ganzhi".to_string(),
                url_or_id: "repo:/huangji_core/src/calendar/ganzhi.rs".to_string(),
                version: None,
            },
            AuthorityEvidenceRef {
                label: "huangji_core::astro::solar".to_string(),
                url_or_id: "repo:/huangji_core/src/astro/solar.rs".to_string(),
                version: None,
            },
        ],
    }
}

fn canonical_result(payload: Value, version: Option<String>) -> EngineResult {
    EngineResult {
        source: BaziSource::Sxtwl,
        payload,
        authority_level: "canonical",
        evidence_refs: engine_evidence(BaziSource::Sxtwl, version),
    }
}

fn legacy_result(payload: Value) -> EngineResult {
    EngineResult {
        source: BaziSource::HuangjiCore,
        payload,
        authority_level: "derived",
        evidence_refs: engine_evidence(BaziSource::HuangjiCore, None),
    }
}

fn as_api_error(error: &str, message: impl Into<String>) -> ApiError {
    (
        StatusCode::INTERNAL_SERVER_ERROR,
        Json(json!({
            "error": error,
            "message": message.into(),
        })),
    )
}

fn bridge_failure_to_variant(failure: &BridgeFailure) -> BaziVariant {
    BaziVariant::unavailable(format!("{}: {}", failure.reason, failure.detail))
}

pub async fn resolve_bazi(ctx: &BaziRequestContext) -> Result<Value, ApiError> {
    let mut sxtwl_variant = BaziVariant::unavailable("not_requested");
    let legacy_variant;

    let mut sxtwl_result: Option<EngineResult> = None;
    let mut sxtwl_failure: Option<BridgeFailure> = None;

    if !matches!(ctx.requested_source, BaziSource::HuangjiCore) {
        match compute_with_sxtwl(ctx).await {
            Ok(success) => {
                let payload = legacy::build_payload_from_pillars(ctx, &success.pillars);
                sxtwl_variant = BaziVariant::available(payload.clone());
                sxtwl_result = Some(canonical_result(payload, success.engine_version));
            }
            Err(error) => {
                sxtwl_variant = bridge_failure_to_variant(&error);
                sxtwl_failure = Some(error);
            }
        }
    }

    let legacy_payload = legacy::build_legacy_payload(ctx);
    legacy_variant = BaziVariant::available(legacy_payload.clone());
    let legacy_result = legacy_result(legacy_payload);

    let (resolved, fallback_reason) = match ctx.requested_source {
        BaziSource::Auto | BaziSource::Sxtwl => {
            if let Some(result) = sxtwl_result {
                (result, None)
            } else {
                let reason = sxtwl_failure
                    .as_ref()
                    .map(|failure| failure.reason.clone())
                    .unwrap_or_else(|| "primary_unavailable".to_string());
                (legacy_result, Some(reason))
            }
        }
        BaziSource::HuangjiCore => (legacy_result, None),
    };

    let authority = AuthorityMeta {
        requested_source: ctx.requested_source.as_str().to_string(),
        resolved_source: resolved.source.as_str().to_string(),
        fallback_reason,
        authority_level: resolved.authority_level.to_string(),
        rule_profile: ctx.rule_profile(),
        evidence_refs: resolved.evidence_refs,
    };

    let variants = BaziVariants {
        sxtwl: sxtwl_variant,
        huangji_core: legacy_variant,
    };

    let mut response = resolved.payload;
    let obj = response
        .as_object_mut()
        .ok_or_else(|| as_api_error("invalid_payload", "resolved payload is not an object"))?;
    obj.insert(
        "authority".to_string(),
        serde_json::to_value(&authority).map_err(|error| {
            as_api_error(
                "serialize_authority_failed",
                format!("serialize authority failed: {}", error),
            )
        })?,
    );
    obj.insert(
        "variants".to_string(),
        serde_json::to_value(&variants).map_err(|error| {
            as_api_error(
                "serialize_variants_failed",
                format!("serialize variants failed: {}", error),
            )
        })?,
    );

    Ok(response)
}
