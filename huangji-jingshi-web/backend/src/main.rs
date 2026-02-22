use axum::response::IntoResponse;
use axum::{
    extract::{Path, Query},
    http::StatusCode,
    routing::{get, post},
    Json, Router,
};
use chrono::{DateTime, Datelike, FixedOffset, NaiveDateTime, TimeZone, Utc};
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::collections::HashMap;
use std::env;
use std::path::PathBuf;
use std::sync::RwLock;
use tower_http::cors::CorsLayer;

mod bazi;

// 使用 huangji_core 公共模块（天文/历法/八字计算）
use huangji_core::calendar::time_rule::{utc_to_hj_year, YearStartMode};
// use huangji_core::algorithm::year_to_acc;
use huangji_core::algorithm;
use huangji_core::fortune::{compute_fortune, CalcMode, FortuneRequest, PrimaryMode};
use huangji_core::huangji_table;
use huangji_core::sky::{compute_sky, SkyRequest};
use huangji_core::table_engine;

// 静态数据缓存
static TIMELINE_DATA: Lazy<RwLock<HashMap<i32, serde_json::Value>>> =
    Lazy::new(|| RwLock::new(HashMap::new()));

static HISTORY_DATA: Lazy<RwLock<serde_json::Value>> =
    Lazy::new(|| RwLock::new(serde_json::Value::Null));

#[derive(Debug, Clone, Copy, Default, Serialize)]
struct DataSourceStatus {
    major_events_loaded: bool,
    history_loaded: bool,
}

static DATA_SOURCE_STATUS: Lazy<RwLock<DataSourceStatus>> =
    Lazy::new(|| RwLock::new(DataSourceStatus::default()));

static CELESTIAL_HASHES: Lazy<RwLock<HashMap<String, String>>> =
    Lazy::new(|| RwLock::new(HashMap::new()));

static SKY_SETTINGS: Lazy<RwLock<serde_json::Value>> = Lazy::new(|| {
    RwLock::new(json!({
        "default_lat": 39.9,
        "default_lon": 116.4,
        "show_stars": true,
        "show_constellations": true,
        "show_planets": true,
        "chinese_labels": true,
        "huangji_mode": true
    }))
});

#[tokio::main]
async fn main() {
    // 初始化 logging
    let default_log_level = "info";
    env::set_var("RUST_LOG", default_log_level);
    tracing_subscriber::fmt::init();

    tracing::info!("🚀 皇极经世后端服务启动中...");

    // 设置端口
    let port = env::var("PORT")
        .unwrap_or_else(|_| "3000".to_string())
        .parse::<u16>()
        .unwrap_or(3000);

    // 智能路径检测
    let data_path = find_data_path();
    tracing::info!("📁 数据路径: {:?}", data_path);

    // 初始化数据加载（禁止静默 Mock，缺数据直接失败）
    let path = data_path.unwrap_or_else(|| {
        panic!("未找到数据文件，服务终止。请确保 data/celestial 目录存在或配置正确。");
    });
    tracing::info!("📂 尝试加载数据文件...");
    if let Err(err) = load_data_files(&path).await {
        panic!("加载数据文件失败: {}", err);
    }
    bazi::log_sxtwl_health();

    // 创建路由
    let app = Router::new()
        // 健康检查 - 立即响应
        .route("/health", get(health_check))
        .route("/", get(root_handler))
        // 核心 API 路由
        .route("/api/sky-and-fortune", get(get_sky_and_fortune))
        .route("/api/calculate", post(calculate))
        .route("/api/timeline", get(get_timeline))
        .route("/api/history", get(get_history))
        .route("/api/history/related", get(get_history_related))
        .route("/api/mapping/get", get(get_mapping))
        .route("/api/celestial/hashes", get(get_celestial_hashes))
        .route("/api/sky/settings", get(get_sky_settings))
        .route("/api/sky/settings", post(update_sky_settings))
        .route("/api/settings/sky", get(get_sky_settings))
        .route("/api/settings/sky", post(update_sky_settings))
        // 八字排盘 API
        .route("/api/bazi", get(bazi::get_bazi))
        // 地理位置服务（代理，解决大陆访问问题）
        .route("/api/geocode/reverse", get(reverse_geocode))
        .route("/api/geocode", get(geocode))
        .route("/api/geoip", get(get_geoip))
        // 静态文件服务
        // axum 0.8 uses matchit syntax: path params are `{name}` (not `:name`)
        .route("/static/{file}", get(static_handler))
        // CORS - 允许所有来源
        .layer(CorsLayer::permissive());

    tracing::info!("🌐 启动服务器，端口: {}", port);

    // 启动服务器
    let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{}", port))
        .await
        .unwrap();
    axum::serve(listener, app).await.unwrap();
}

// 智能路径检测函数
fn find_data_path() -> Option<PathBuf> {
    let possible_paths = [
        "data/celestial",
        "backend/data/celestial",
        "../data/celestial",
        "./data/celestial",
        "/opt/render/project/src/data/celestial",
        "/workspace/data/celestial",
        "/app/data/celestial",
    ];

    tracing::info!("🔍 搜索数据文件...");
    for path in &possible_paths {
        let p = PathBuf::from(path);
        if p.exists() {
            tracing::info!("✅ 找到数据路径: {}", path);
            return Some(p);
        }
        tracing::debug!("❌ 路径不存在: {}", path);
    }

    None
}

fn parse_year(value: &serde_json::Value) -> Option<i32> {
    match value {
        serde_json::Value::Number(num) => i32::try_from(num.as_i64()?).ok(),
        serde_json::Value::String(text) => text.trim().parse::<i32>().ok(),
        _ => None,
    }
}

fn ensure_year(event: &serde_json::Value, year: i32) -> Option<serde_json::Value> {
    let serde_json::Value::Object(map) = event else {
        return None;
    };

    let mut patched = map.clone();
    if !patched.contains_key("year") {
        patched.insert(
            "year".to_string(),
            serde_json::Value::Number(serde_json::Number::from(year)),
        );
    }
    Some(serde_json::Value::Object(patched))
}

fn extract_events_array(value: &serde_json::Value) -> Vec<serde_json::Value> {
    match value {
        serde_json::Value::Array(items) => items.clone(),
        serde_json::Value::Object(obj) => {
            if let Some(events) = obj.get("events").and_then(|events| events.as_array()) {
                return events.clone();
            }

            let mut events = Vec::new();
            for (key, raw_value) in obj {
                let Ok(year) = key.parse::<i32>() else {
                    continue;
                };
                match raw_value {
                    serde_json::Value::Array(items) => {
                        for item in items {
                            if let Some(event) = ensure_year(item, year) {
                                events.push(event);
                            }
                        }
                    }
                    serde_json::Value::Object(_) => {
                        if let Some(event) = ensure_year(raw_value, year) {
                            events.push(event);
                        }
                    }
                    _ => {}
                }
            }
            events
        }
        _ => Vec::new(),
    }
}

fn index_events_by_year(events: &[serde_json::Value]) -> HashMap<i32, serde_json::Value> {
    let mut grouped: HashMap<i32, Vec<serde_json::Value>> = HashMap::new();
    for event in events {
        let Some(year) = event.get("year").and_then(parse_year) else {
            continue;
        };
        grouped.entry(year).or_default().push(event.clone());
    }

    grouped
        .into_iter()
        .map(|(year, grouped_events)| (year, serde_json::Value::Array(grouped_events)))
        .collect()
}

// 数据加载函数
async fn load_data_files(data_path: &PathBuf) -> Result<(), Box<dyn std::error::Error>> {
    tracing::info!("📊 开始加载数据文件...");

    // 获取数据根目录
    let data_root = if data_path.to_str().unwrap().contains("celestial") {
        data_path.parent().unwrap_or(data_path)
    } else {
        data_path
    };

    let mut history_events = Vec::new();
    let mut major_events = Vec::new();
    let mut status = DataSourceStatus::default();

    // reset stale cache before loading
    *TIMELINE_DATA.write().unwrap() = HashMap::new();
    *HISTORY_DATA.write().unwrap() = serde_json::Value::Array(Vec::new());

    // 加载历史数据
    let history_path = data_root.join("history.json");
    tracing::info!("🔍 尝试加载历史数据: {:?}", history_path);
    if history_path.exists() {
        match load_json_file(&history_path).await {
            Ok(data) => {
                history_events = extract_events_array(&data);
                status.history_loaded = !history_events.is_empty();
                tracing::info!("✅ 历史数据加载成功: {} 条", history_events.len());
            }
            Err(e) => tracing::warn!("⚠️ 历史数据加载失败: {}", e),
        }
    }

    // 加载主要事件数据
    let major_events_path = data_root.join("major_events.json");
    tracing::info!("🔍 尝试加载事件数据: {:?}", major_events_path);
    if major_events_path.exists() {
        match load_json_file(&major_events_path).await {
            Ok(data) => {
                major_events = extract_events_array(&data);
                status.major_events_loaded = !major_events.is_empty();
                let indexed = index_events_by_year(&major_events);
                let year_count = indexed.len();
                *TIMELINE_DATA.write().unwrap() = indexed;
                tracing::info!(
                    "✅ 主要事件数据加载成功: {} 条, 覆盖 {} 年",
                    major_events.len(),
                    year_count
                );
            }
            Err(e) => tracing::warn!("⚠️ 主要事件数据加载失败: {}", e),
        }
    }

    if history_events.is_empty() && !major_events.is_empty() {
        tracing::info!(
            "ℹ️ history.json 为空，回退使用 major_events.json: {} 条",
            major_events.len()
        );
        history_events = major_events.clone();
    }

    *HISTORY_DATA.write().unwrap() = serde_json::Value::Array(history_events);
    *DATA_SOURCE_STATUS.write().unwrap() = status;

    tracing::info!("🎯 数据文件加载完成");
    Ok(())
}

// JSON文件加载
async fn load_json_file(path: &PathBuf) -> Result<serde_json::Value, Box<dyn std::error::Error>> {
    let content = tokio::fs::read_to_string(path).await?;
    Ok(serde_json::from_str(&content)?)
}

// 健康检查 - 立即响应
async fn health_check() -> impl IntoResponse {
    tracing::debug!("💊 健康检查请求");
    let timeline_years_loaded = TIMELINE_DATA.read().unwrap().len();
    let history_events_loaded = {
        let history = HISTORY_DATA.read().unwrap();
        extract_events_array(&history).len()
    };
    let data_sources = *DATA_SOURCE_STATUS.read().unwrap();
    let data_loaded = timeline_years_loaded > 0 || history_events_loaded > 0;
    Json(json!({
        "status": "ok",
        "message": "皇极经世后端服务正常运行",
        "timestamp": Utc::now().to_rfc3339(),
        "version": "1.0.0-fixed",
        "data_loaded": data_loaded,
        "timeline_years_loaded": timeline_years_loaded,
        "history_events_loaded": history_events_loaded,
        "data_sources": data_sources
    }))
}

// 根路径处理器
async fn root_handler() -> impl IntoResponse {
    Json(json!({
        "service": "皇极经世后端服务",
        "status": "running",
        "version": "1.1.0",
        "message": "API服务正常运行",
        "endpoints": [
            "GET /health",
            "GET /api/sky-and-fortune",
            "POST /api/calculate",
            "GET /api/timeline",
            "GET /api/history",
            "GET /api/history/related",
            "GET /api/mapping/get",
            "GET /api/celestial/hashes",
            "GET /api/sky/settings",
            "POST /api/sky/settings",
            "GET /api/settings/sky",
            "POST /api/settings/sky"
        ]
    }))
}

// 天机演算（禁止 Mock，尚未实现则返回 501）
async fn calculate(Json(payload): Json<serde_json::Value>) -> impl IntoResponse {
    tracing::info!("🔮 收到演算请求: {:?}", payload);

    (
        StatusCode::NOT_IMPLEMENTED,
        Json(json!({
            "error": "演算功能尚未实现",
            "message": "请使用真实演算实现后再调用此接口",
            "input": payload,
            "timestamp": Utc::now().to_rfc3339(),
            "status": "not_implemented"
        })),
    )
}

#[derive(Deserialize)]
struct TimelineQuery {
    datetime: String,
    /// 时区偏移（分钟），东为正 UTC+8=+480, 西为负 UTC-5=-300
    /// 注意：与 JS Date.getTimezoneOffset() 符号相反！
    #[serde(rename = "tzOffsetMinutes")]
    tz_offset_minutes: Option<i32>,
    lon: Option<f64>, // 用于真太阳时校正
    #[serde(rename = "yearStart")]
    year_start: Option<String>,
    mode: Option<String>,
    primary: Option<String>,
}

#[derive(Deserialize)]
struct SkyFortuneQuery {
    datetime: String,
    lat: Option<f64>,
    lon: Option<f64>,
    /// 时区偏移（分钟），东为正 UTC+8=+480, 西为负 UTC-5=-300
    /// 注意：与 JS Date.getTimezoneOffset() 符号相反！
    #[serde(rename = "tzOffsetMinutes")]
    tz_offset_minutes: Option<i32>,
    /// 是否使用真太阳时（可选）
    #[serde(rename = "useTrueSolarTime")]
    use_true_solar_time: Option<bool>,
    #[serde(rename = "yearStart")]
    year_start: Option<String>,
    mode: Option<String>,
    primary: Option<String>,
}

// HistoryQuery 保留用于将来的历史数据过滤
#[allow(dead_code)]
#[derive(Deserialize)]
struct HistoryQuery {
    start: Option<i32>,
    end: Option<i32>,
}

#[derive(Deserialize)]
struct HistoryRelatedQuery {
    year: Option<i32>,
    #[allow(dead_code)]
    mode: Option<String>, // 保留用于将来的查询模式
    limit: Option<i32>,
}

#[derive(Deserialize)]
struct MappingQuery {
    year: Option<i32>,
    #[serde(rename = "yearStart")]
    year_start: Option<String>,
    mode: Option<String>,
    primary: Option<String>,
}

#[derive(Deserialize)]
struct GeocodeQuery {
    lat: f64,
    lon: f64,
}

#[derive(Deserialize)]
struct GeocodeForwardQuery {
    address: String,
}

fn parse_calc_mode(input: Option<&str>) -> CalcMode {
    input
        .and_then(|value| value.parse::<CalcMode>().ok())
        .unwrap_or_default()
}

fn parse_primary_mode(input: Option<&str>) -> PrimaryMode {
    input
        .and_then(|value| value.parse::<PrimaryMode>().ok())
        .unwrap_or_default()
}

fn parse_year_start_mode(input: Option<&str>) -> YearStartMode {
    match input
        .unwrap_or("lichun")
        .trim()
        .to_ascii_lowercase()
        .as_str()
    {
        "gregorian" => YearStartMode::GregorianNewYear,
        _ => YearStartMode::Lichun,
    }
}

fn parse_datetime_or_bad_request(
    raw: &str,
    tz_offset_minutes: i32,
) -> Result<DateTime<Utc>, (StatusCode, Json<serde_json::Value>)> {
    if let Ok(dt) = DateTime::parse_from_rfc3339(raw) {
        return Ok(dt.with_timezone(&Utc));
    }

    let naive = NaiveDateTime::parse_from_str(raw, "%Y-%m-%dT%H:%M:%S")
        .or_else(|_| NaiveDateTime::parse_from_str(raw, "%Y-%m-%dT%H:%M"));

    if let Ok(naive) = naive {
        let Some(offset) = FixedOffset::east_opt(tz_offset_minutes * 60) else {
            return Err((
                StatusCode::BAD_REQUEST,
                Json(json!({
                    "error": "invalid_tz_offset",
                    "message": format!(
                        "invalid tzOffsetMinutes '{}' for datetime '{}'",
                        tz_offset_minutes, raw
                    ),
                })),
            ));
        };

        let Some(local_dt) = offset.from_local_datetime(&naive).single() else {
            return Err((
                StatusCode::BAD_REQUEST,
                Json(json!({
                    "error": "invalid_local_datetime",
                    "message": format!(
                        "cannot resolve local datetime '{}' with tzOffsetMinutes '{}'",
                        raw, tz_offset_minutes
                    ),
                })),
            ));
        };

        return Ok(local_dt.with_timezone(&Utc));
    }

    Err((
        StatusCode::BAD_REQUEST,
        Json(json!({
            "error": "invalid_datetime",
            "message": format!(
                "invalid datetime '{}', expected RFC3339 or local ISO format (YYYY-MM-DDTHH:MM[:SS])",
                raw
            ),
        })),
    ))
}

fn parse_query_datetime(
    raw: &str,
    tz_offset_minutes: i32,
) -> Result<DateTime<Utc>, (StatusCode, Json<serde_json::Value>)> {
    let parsed = parse_datetime_or_bad_request(raw, tz_offset_minutes);
    if let Err((status, body)) = &parsed {
        tracing::warn!(
            "❌ 日期时间解析失败: datetime={}, tzOffsetMinutes={}, status={}, error={:?}",
            raw,
            tz_offset_minutes,
            status,
            body.0
        );
    }
    parsed
}

fn source_label(primary: PrimaryMode) -> &'static str {
    if matches!(primary, PrimaryMode::Table) {
        "table"
    } else {
        "algorithm"
    }
}

// 核心 API - 获取天象和运势数据
async fn get_sky_and_fortune(
    Query(params): Query<SkyFortuneQuery>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    let tz_offset_minutes = params.tz_offset_minutes.unwrap_or(480);
    let lat = params.lat.unwrap_or(39.9);
    let lon = params.lon.unwrap_or(116.4);
    let use_true_solar_time = params.use_true_solar_time.unwrap_or(false);
    let mode = parse_calc_mode(params.mode.as_deref());
    let primary = parse_primary_mode(params.primary.as_deref());
    let year_start = parse_year_start_mode(params.year_start.as_deref());

    tracing::info!(
        "🌟 获取天象运势: datetime={}, tzOffsetMinutes={}, useTrueSolarTime={}, lat={}, lon={}, mode={:?}, primary={:?}, yearStart={:?}",
        params.datetime,
        tz_offset_minutes,
        use_true_solar_time,
        lat,
        lon,
        mode,
        primary,
        year_start
    );

    // 解析输入时间：优先 RFC3339（带 Z 或 offset），否则按“本地时间 + tzOffsetMinutes”解释
    let datetime_utc = parse_query_datetime(&params.datetime, tz_offset_minutes)?;

    let sky_resp = compute_sky(&SkyRequest {
        datetime: datetime_utc,
        lat_deg: lat,
        lon_deg: lon,
        delta_t_provider: None,
        accuracy: None,
    });

    let fortune_resp = compute_fortune(&FortuneRequest {
        datetime: datetime_utc,
        tz_offset_minutes: Some(tz_offset_minutes),
        lon: Some(lon),
        use_true_solar_time: Some(use_true_solar_time),
        mode: Some(mode),
        year_start: Some(year_start),
        primary: Some(primary),
    });

    Ok(Json(json!({
        "sky": sky_resp,
        "fortune": fortune_resp
    })))
}

// 获取历史相关事件 - 返回纯数组，不是对象
async fn get_history_related(Query(params): Query<HistoryRelatedQuery>) -> impl IntoResponse {
    let year = params.year.unwrap_or(2025);
    let _limit = params.limit.unwrap_or(3);

    tracing::debug!("📚 获取相关历史: year={}, limit={}", year, _limit);

    // 直接返回数组，不要包装在 { events: [...] } 中
    Json(json!([
        {"year": year - 60, "title": "甲子年事件", "dynasty": "近代", "person": ""},
        {"year": year - 120, "title": "往年大事", "dynasty": "清朝", "person": ""},
        {"year": year - 180, "title": "古代记录", "dynasty": "清朝", "person": ""}
    ]))
}

// 获取映射记录
async fn get_mapping(Query(params): Query<MappingQuery>) -> impl IntoResponse {
    let year = params.year.unwrap_or_else(|| Utc::now().year());
    let mode = parse_calc_mode(params.mode.as_deref());
    let primary = parse_primary_mode(params.primary.as_deref());
    let year_start = parse_year_start_mode(params.year_start.as_deref());

    tracing::debug!(
        "🗺️ 获取映射记录: year={}, mode={:?}, primary={:?}, year_start={:?}",
        year,
        mode,
        primary,
        year_start
    );

    let record_raw = huangji_table::get_year_record(year);
    let record_normalized = huangji_table::get_year_record_normalized(year);
    let coverage = huangji_table::get_coverage();

    let (available, reason) = if record_raw.is_some() {
        (true, "record found".to_string())
    } else if let Some(range) = coverage {
        (
            false,
            format!(
                "record not found, table coverage {}-{}",
                range.min_year, range.max_year
            ),
        )
    } else {
        (false, "record not found".to_string())
    };

    Json(json!({
        "year": year,
        "mode": mode,
        "primary": primary,
        "year_start": match year_start {
            YearStartMode::Lichun => "lichun",
            YearStartMode::GregorianNewYear => "gregorian",
        },
        "available": available,
        "reason": reason,
        "record_raw": record_raw,
        "record_normalized": record_normalized
    }))
}

// 获取时间线
async fn get_timeline(
    Query(params): Query<TimelineQuery>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    let tz_offset_minutes = params.tz_offset_minutes.unwrap_or(480);
    let lon = params.lon.unwrap_or(116.4);
    let mode = parse_calc_mode(params.mode.as_deref());
    let primary = parse_primary_mode(params.primary.as_deref());
    let year_start = parse_year_start_mode(params.year_start.as_deref());

    let datetime_utc = parse_query_datetime(&params.datetime, tz_offset_minutes)?;

    let fallback_hj_year = utc_to_hj_year(datetime_utc, tz_offset_minutes, lon, false, year_start);

    tracing::debug!(
        "📅 查询时间线: mode={:?}, primary={:?}, year_start={:?}",
        mode,
        primary,
        year_start
    );

    let fortune = compute_fortune(&FortuneRequest {
        datetime: datetime_utc,
        tz_offset_minutes: Some(tz_offset_minutes),
        lon: Some(lon),
        use_true_solar_time: Some(false),
        mode: Some(mode),
        year_start: Some(year_start),
        primary: Some(primary),
    });

    let hj_year = fortune
        .calc_meta
        .as_ref()
        .map(|meta| meta.hj_year)
        .unwrap_or(fallback_hj_year);

    tracing::debug!("📅 时间线经世年: {}", hj_year);

    let algorithm_timeline = algorithm::get_timeline_info(hj_year);
    let table_timeline = table_engine::get_timeline_info(hj_year);
    let resolved_primary = fortune
        .calc_meta
        .as_ref()
        .map(|meta| meta.primary)
        .unwrap_or(PrimaryMode::Algorithm);

    let primary_timeline = if matches!(resolved_primary, PrimaryMode::Table) {
        table_timeline
            .clone()
            .unwrap_or_else(|| algorithm_timeline.clone())
    } else {
        algorithm_timeline.clone()
    };

    let secondary_source = if matches!(resolved_primary, PrimaryMode::Table) {
        Some("algorithm")
    } else if table_timeline.is_some() {
        Some("table")
    } else {
        None
    };

    Ok(Json(json!({
        "year": hj_year,
        "current": primary_timeline.current,
        "yuan_list": primary_timeline.yuan_list,
        "hui_list": primary_timeline.hui_list,
        "yun_list": primary_timeline.yun_list,
        "shi_list": primary_timeline.shi_list,
        "xun_list": primary_timeline.xun_list,
        "calc_meta": fortune.calc_meta,
        "variants": fortune.variants,
        "diff": fortune.diff,
        "mapping_record": fortune.mapping_record,
        "authority": fortune.authority,
        "timeline_meta": {
            "primary_source": source_label(resolved_primary),
            "secondary_source": secondary_source,
        },
        "timeline_variants": {
            "algorithm": algorithm_timeline,
            "table": table_timeline
        }
    })))
}

// 获取历史数据 - 返回数组格式
async fn get_history() -> impl IntoResponse {
    let data = HISTORY_DATA.read().unwrap().clone();
    Json(serde_json::Value::Array(extract_events_array(&data)))
}

// 获取天体哈希
async fn get_celestial_hashes() -> impl IntoResponse {
    Json(serde_json::to_value(&*CELESTIAL_HASHES.read().unwrap()).unwrap())
}

// 获取天空设置
async fn get_sky_settings() -> impl IntoResponse {
    Json(SKY_SETTINGS.read().unwrap().clone())
}

// 更新天空设置
async fn update_sky_settings(Json(settings): Json<serde_json::Value>) -> impl IntoResponse {
    tracing::info!("🔧 更新天空设置: {:?}", settings);
    *SKY_SETTINGS.write().unwrap() = settings;
    Json(json!({ "status": "success", "message": "设置已更新" }))
}

// 静态文件服务
async fn static_handler(Path(file): Path<String>) -> impl IntoResponse {
    let file_path = format!("static/{}", file);
    if let Ok(content) = tokio::fs::read_to_string(&file_path).await {
        Json(json!({ "content": content }))
    } else {
        Json(json!({ "error": "File not found", "file": file }))
    }
}

// ==================== 八字排盘 API ====================
// moved to backend/src/bazi/*

// ==================== 地理位置服务 ====================

// 地理编码：地址转经纬度
async fn geocode(Query(params): Query<GeocodeForwardQuery>) -> impl IntoResponse {
    tracing::debug!("🗺️ 地理编码请求: address={}", params.address);

    let address = params.address.trim();
    if address.is_empty() {
        return Json(json!({
            "error": "地址不能为空"
        }));
    }

    // 方法1: OpenStreetMap Nominatim (支持中文地址)
    if let Ok(client) = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(8))
        .user_agent("HuangjiJingshiWeb/1.0")
        .build()
    {
        let url = format!(
            "https://nominatim.openstreetmap.org/search?q={}&format=json&limit=1&accept-language=zh-CN",
            urlencoding::encode(address)
        );

        if let Ok(res) = client.get(&url).send().await {
            if let Ok(data) = res.json::<Vec<serde_json::Value>>().await {
                if let Some(first) = data.first() {
                    if let (Some(lat), Some(lon)) = (
                        first["lat"].as_str().and_then(|s| s.parse::<f64>().ok()),
                        first["lon"].as_str().and_then(|s| s.parse::<f64>().ok()),
                    ) {
                        let display_name = first["display_name"].as_str().unwrap_or(address);
                        return Json(json!({
                            "latitude": lat,
                            "longitude": lon,
                            "address": display_name,
                            "source": "OpenStreetMap"
                        }));
                    }
                }
            }
        }
    }

    // 方法2: BigDataCloud (备用，对中国地址支持有限)
    if let Ok(client) = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(8))
        .build()
    {
        let url = format!(
            "https://api.bigdatacloud.net/data/forward-geocode-client?query={}&localityLanguage=zh",
            urlencoding::encode(address)
        );

        if let Ok(res) = client.get(&url).send().await {
            if let Ok(data) = res.json::<serde_json::Value>().await {
                if let Some(results) = data["results"].as_array() {
                    if let Some(first) = results.first() {
                        if let (Some(lat), Some(lon)) =
                            (first["latitude"].as_f64(), first["longitude"].as_f64())
                        {
                            let formatted = first["formatted"].as_str().unwrap_or(address);
                            return Json(json!({
                                "latitude": lat,
                                "longitude": lon,
                                "address": formatted,
                                "source": "BigDataCloud"
                            }));
                        }
                    }
                }
            }
        }
    }

    // 都失败了
    Json(json!({
        "error": "无法找到该地址的坐标，请检查地址是否正确或手动输入经纬度",
        "source": "none"
    }))
}

// 逆地理编码：经纬度转地名
async fn reverse_geocode(Query(params): Query<GeocodeQuery>) -> impl IntoResponse {
    tracing::debug!("🗺️ 逆地理编码请求: lat={}, lon={}", params.lat, params.lon);

    // 尝试多个服务，提高成功率

    // 方法1: BigDataCloud (免费，无需密钥，大陆可访问)
    if let Ok(client) = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(5))
        .build()
    {
        let url = format!(
            "https://api.bigdatacloud.net/data/reverse-geocode-client?latitude={}&longitude={}&localityLanguage=zh",
            params.lat, params.lon
        );

        if let Ok(res) = client.get(&url).send().await {
            if let Ok(data) = res.json::<serde_json::Value>().await {
                let location = data["city"]
                    .as_str()
                    .or(data["locality"].as_str())
                    .or(data["principalSubdivision"].as_str())
                    .or(data["countryName"].as_str())
                    .unwrap_or("未知地点");

                return Json(json!({
                    "location": location,
                    "source": "BigDataCloud"
                }));
            }
        }
    }

    // 方法2: OpenStreetMap Nominatim (备用)
    if let Ok(client) = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(5))
        .user_agent("HuangjiJingshiWeb/1.0")
        .build()
    {
        let url = format!(
            "https://nominatim.openstreetmap.org/reverse?lat={}&lon={}&format=json&accept-language=zh-CN",
            params.lat, params.lon
        );

        if let Ok(res) = client.get(&url).send().await {
            if let Ok(data) = res.json::<serde_json::Value>().await {
                if let Some(address) = data.get("address") {
                    let location = address["city"]
                        .as_str()
                        .or(address["town"].as_str())
                        .or(address["county"].as_str())
                        .or(address["state"].as_str())
                        .unwrap_or("未知地点");

                    return Json(json!({
                        "location": location,
                        "source": "OpenStreetMap"
                    }));
                }
            }
        }
    }

    // 都失败了
    Json(json!({
        "location": "未知地点",
        "source": "fallback"
    }))
}

// IP 地理定位
async fn get_geoip() -> impl IntoResponse {
    tracing::debug!("🌐 IP定位请求");

    // 尝试多个IP定位服务

    // 方法1: ip-api.com (免费，大陆可访问)
    if let Ok(client) = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(5))
        .build()
    {
        let url = "http://ip-api.com/json/?lang=zh-CN";

        if let Ok(res) = client.get(url).send().await {
            if let Ok(data) = res.json::<serde_json::Value>().await {
                if data["status"].as_str() == Some("success") {
                    return Json(json!({
                        "latitude": data["lat"].as_f64().unwrap_or(39.9),
                        "longitude": data["lon"].as_f64().unwrap_or(116.4),
                        "city": data["city"].as_str().unwrap_or("北京"),
                        "region": data["regionName"].as_str().unwrap_or("北京市"),
                        "country": data["country"].as_str().unwrap_or("中国"),
                        "source": "ip-api.com"
                    }));
                }
            }
        }
    }

    // 方法2: ipapi.co (备用)
    if let Ok(client) = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(5))
        .build()
    {
        let url = "https://ipapi.co/json/";

        if let Ok(res) = client.get(url).send().await {
            if let Ok(data) = res.json::<serde_json::Value>().await {
                return Json(json!({
                    "latitude": data["latitude"].as_f64().unwrap_or(39.9),
                    "longitude": data["longitude"].as_f64().unwrap_or(116.4),
                    "city": data["city"].as_str().unwrap_or("北京"),
                    "region": data["region"].as_str().unwrap_or("北京市"),
                    "country": data["country_name"].as_str().unwrap_or("中国"),
                    "source": "ipapi.co"
                }));
            }
        }
    }

    // 都失败了，返回默认北京坐标
    Json(json!({
        "latitude": 39.9042,
        "longitude": 116.4074,
        "city": "北京",
        "region": "北京市",
        "country": "中国",
        "source": "fallback"
    }))
}

#[cfg(test)]
mod tests {
    use super::{extract_events_array, index_events_by_year, parse_query_datetime};
    use serde_json::json;

    #[test]
    fn extract_events_array_accepts_direct_array() {
        let input = json!([
            {"year": 2025, "title": "A"},
            {"year": 2026, "title": "B"}
        ]);
        let events = extract_events_array(&input);
        assert_eq!(events.len(), 2);
    }

    #[test]
    fn extract_events_array_accepts_events_object() {
        let input = json!({
            "events": [
                {"year": 2025, "title": "A"}
            ]
        });
        let events = extract_events_array(&input);
        assert_eq!(events.len(), 1);
        assert_eq!(events[0]["year"], json!(2025));
    }

    #[test]
    fn extract_events_array_accepts_year_map_and_backfills_year() {
        let input = json!({
            "2025": {"title": "single"},
            "2026": [
                {"title": "multi-a"},
                {"year": 2026, "title": "multi-b"}
            ]
        });
        let events = extract_events_array(&input);
        assert_eq!(events.len(), 3);

        let mut year_by_title = std::collections::HashMap::new();
        for event in events {
            if let (Some(title), Some(year)) = (event["title"].as_str(), event["year"].as_i64()) {
                year_by_title.insert(title.to_string(), year);
            }
        }

        assert_eq!(year_by_title.get("single"), Some(&2025));
        assert_eq!(year_by_title.get("multi-a"), Some(&2026));
        assert_eq!(year_by_title.get("multi-b"), Some(&2026));
    }

    #[test]
    fn index_events_by_year_groups_multiple_events_in_one_year() {
        let events = vec![
            json!({"year": 2025, "title": "A"}),
            json!({"year": 2025, "title": "B"}),
            json!({"year": 2026, "title": "C"}),
        ];
        let indexed = index_events_by_year(&events);

        assert_eq!(indexed.len(), 2);
        assert_eq!(
            indexed
                .get(&2025)
                .and_then(|v| v.as_array())
                .map(std::vec::Vec::len),
            Some(2)
        );
        assert_eq!(
            indexed
                .get(&2026)
                .and_then(|v| v.as_array())
                .map(std::vec::Vec::len),
            Some(1)
        );
    }

    #[test]
    fn parse_query_datetime_accepts_local_iso_with_offset() {
        let parsed = parse_query_datetime("2026-06-01T08:00:00", 480).expect("valid datetime");
        assert_eq!(parsed.to_rfc3339(), "2026-06-01T00:00:00+00:00");
    }

    #[test]
    fn parse_query_datetime_rejects_invalid_input() {
        let parsed = parse_query_datetime("not-a-date", 480);
        assert!(parsed.is_err());
    }
}
