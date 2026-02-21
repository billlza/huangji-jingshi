use axum::response::IntoResponse;
use axum::{
    extract::{Path, Query},
    http::StatusCode,
    routing::{get, post},
    Json, Router,
};
use chrono::{Datelike, TimeZone, Utc};
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::collections::HashMap;
use std::env;
use std::path::PathBuf;
use std::sync::RwLock;
use tower_http::cors::CorsLayer;

// 使用 huangji_core 公共模块（天文/历法/八字计算）
use huangji_core::astro::solar::true_solar_hour;
use huangji_core::calendar::ganzhi::{
    calc_bazi_pillars, calc_dayun_start_age, DIZHI, GAN_WUXING, NAYIN, SHENGXIAO, TIANGAN,
    ZHI_WUXING,
};
use huangji_core::calendar::time_rule::{datetime_to_hj_year, to_rule_datetime, YearStartMode};
// use huangji_core::algorithm::year_to_acc;
use huangji_core::algorithm;
use huangji_core::fortune::{compute_fortune, CalcMode, FortuneRequest, PrimaryMode};
use huangji_core::huangji_table;
use huangji_core::sky::{compute_sky, SkyRequest};

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
        .route("/api/bazi", get(get_bazi))
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
struct BaziQuery {
    datetime: String,
    #[allow(dead_code)]
    timezone: Option<String>, // 保留用于兼容性
    /// 时区偏移（分钟），东为正 UTC+8=+480, 西为负 UTC-5=-300
    /// 注意：与 JS Date.getTimezoneOffset() 符号相反！
    #[serde(rename = "tzOffsetMinutes")]
    tz_offset_minutes: Option<i32>,
    #[allow(dead_code)]
    lat: Option<f64>, // 保留用于地方时校正
    lon: Option<f64>, // 用于真太阳时校正
    gender: Option<String>,
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

// 核心 API - 获取天象和运势数据
async fn get_sky_and_fortune(Query(params): Query<SkyFortuneQuery>) -> impl IntoResponse {
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
    let datetime_utc = if let Ok(dt) = chrono::DateTime::parse_from_rfc3339(&params.datetime) {
        dt.with_timezone(&Utc)
    } else if let Ok(naive) =
        chrono::NaiveDateTime::parse_from_str(&params.datetime, "%Y-%m-%dT%H:%M:%S")
    {
        // 这里将“无时区字符串”按用户传入 tzOffsetMinutes 解释为本地时间，再换算到 UTC
        if let Some(offset) = chrono::FixedOffset::east_opt(tz_offset_minutes * 60) {
            if let Some(local_dt) = offset.from_local_datetime(&naive).single() {
                local_dt.with_timezone(&Utc)
            } else {
                Utc::now()
            }
        } else {
            Utc::now()
        }
    } else {
        Utc::now()
    };

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

    Json(json!({
        "sky": sky_resp,
        "fortune": fortune_resp
    }))
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
async fn get_timeline(Query(params): Query<TimelineQuery>) -> impl IntoResponse {
    let tz_offset_minutes = params.tz_offset_minutes.unwrap_or(480);
    let lon = params.lon.unwrap_or(116.4);
    let mode = parse_calc_mode(params.mode.as_deref());
    let primary = parse_primary_mode(params.primary.as_deref());
    let year_start = parse_year_start_mode(params.year_start.as_deref());

    let datetime_utc = if let Ok(dt) = chrono::DateTime::parse_from_rfc3339(&params.datetime) {
        dt.with_timezone(&Utc)
    } else if let Ok(naive) =
        chrono::NaiveDateTime::parse_from_str(&params.datetime, "%Y-%m-%dT%H:%M:%S")
    {
        if let Some(offset) = chrono::FixedOffset::east_opt(tz_offset_minutes * 60) {
            if let Some(local_dt) = offset.from_local_datetime(&naive).single() {
                local_dt.with_timezone(&Utc)
            } else {
                Utc::now()
            }
        } else {
            Utc::now()
        }
    } else {
        Utc::now()
    };

    let rule_dt = to_rule_datetime(datetime_utc, tz_offset_minutes, lon, false);
    let hj_year = datetime_to_hj_year(rule_dt, year_start);

    tracing::debug!(
        "📅 查询时间线: hj_year={}, mode={:?}, primary={:?}, year_start={:?}",
        hj_year,
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

    let mut timeline = algorithm::get_timeline_info(hj_year);
    timeline.current.year_gua = fortune.hexagram_major.clone();
    timeline.current.yuan.name = fortune.yuan.clone();
    timeline.current.hui.name = fortune.hui.clone();
    timeline.current.yun.name = fortune.yun.clone();
    timeline.current.shi.name = fortune.shi.clone();
    timeline.current.xun.name = fortune.xun.clone();

    Json(json!({
        "year": hj_year,
        "current": timeline.current,
        "yuan_list": timeline.yuan_list,
        "hui_list": timeline.hui_list,
        "yun_list": timeline.yun_list,
        "shi_list": timeline.shi_list,
        "xun_list": timeline.xun_list,
        "calc_meta": fortune.calc_meta,
        "variants": fortune.variants,
        "diff": fortune.diff,
        "mapping_record": fortune.mapping_record
    }))
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
// 常量从 huangji_core::calendar::ganzhi 导入

// 地支藏干表 (Hidden Stems in Earthly Branches)
// 格式: [余气, 中气, 本气] - 有些地支只有本气或本气+余气
const ZHI_CANGGAN: [[&str; 3]; 12] = [
    ["", "", "癸"],     // 子: 癸水
    ["癸", "辛", "己"], // 丑: 己土(本气) 辛金(中气) 癸水(余气)
    ["戊", "丙", "甲"], // 寅: 甲木(本气) 丙火(中气) 戊土(余气)
    ["", "", "乙"],     // 卯: 乙木
    ["癸", "乙", "戊"], // 辰: 戊土(本气) 乙木(中气) 癸水(余气)
    ["戊", "庚", "丙"], // 巳: 丙火(本气) 庚金(中气) 戊土(余气)
    ["己", "", "丁"],   // 午: 丁火(本气) 己土(余气)
    ["丁", "乙", "己"], // 未: 己土(本气) 乙木(中气) 丁火(余气)
    ["戊", "壬", "庚"], // 申: 庚金(本气) 壬水(中气) 戊土(余气)
    ["", "", "辛"],     // 酉: 辛金
    ["丁", "辛", "戊"], // 戌: 戊土(本气) 辛金(中气) 丁火(余气)
    ["甲", "", "壬"],   // 亥: 壬水(本气) 甲木(余气)
];

// 十神计算表 (Ten Gods Table)
// 根据日干与其他天干的关系，返回十神名称
// 阴阳属性: 0,2,4,6,8=阳  1,3,5,7,9=阴
#[allow(clippy::manual_is_multiple_of)]
fn calculate_ten_god(day_gan_idx: usize, target_gan_idx: usize) -> &'static str {
    let day_is_yang = day_gan_idx % 2 == 0;
    let target_is_yang = target_gan_idx % 2 == 0;
    let same_yin_yang = day_is_yang == target_is_yang;

    // 五行关系: 木(0,1) 火(2,3) 土(4,5) 金(6,7) 水(8,9)
    let day_wuxing = day_gan_idx / 2;
    let target_wuxing = target_gan_idx / 2;

    // 计算五行关系
    let relation = (target_wuxing + 5 - day_wuxing) % 5;

    match relation {
        0 => {
            if same_yin_yang {
                "比肩"
            } else {
                "劫财"
            }
        }
        1 => {
            if same_yin_yang {
                "食神"
            } else {
                "伤官"
            }
        }
        2 => {
            if same_yin_yang {
                "偏财"
            } else {
                "正财"
            }
        }
        3 => {
            if same_yin_yang {
                "偏官"
            } else {
                "正官"
            }
        } // 偏官也叫七杀
        4 => {
            if same_yin_yang {
                "偏印"
            } else {
                "正印"
            }
        } // 偏印也叫枭神
        _ => "未知",
    }
}

// 计算地支藏干的十神
fn get_hidden_stems_with_gods(zhi_idx: usize, day_gan_idx: usize) -> Vec<serde_json::Value> {
    let hidden = &ZHI_CANGGAN[zhi_idx];
    let mut result = Vec::new();

    for (i, gan_str) in hidden.iter().enumerate() {
        if !gan_str.is_empty() {
            // 找到天干索引
            if let Some(gan_idx) = TIANGAN.iter().position(|&g| g == *gan_str) {
                let ten_god = calculate_ten_god(day_gan_idx, gan_idx);
                let gan_wuxing = GAN_WUXING[gan_idx];

                // 确定藏干类型和能量
                let (canggan_type, energy) = match i {
                    0 => ("余气", 30),
                    1 => {
                        if hidden[0].is_empty() {
                            ("余气", 30)
                        } else {
                            ("中气", 20)
                        }
                    }
                    2 => ("本气", 50),
                    _ => ("", 0),
                };

                result.push(json!({
                    "gan": gan_str,
                    "gan_wuxing": gan_wuxing,
                    "ten_god": ten_god,
                    "type": canggan_type,
                    "energy": energy
                }));
            }
        }
    }

    result
}

// 计算大运 (Great Luck Cycles)
fn calculate_dayun(
    month_gan_idx: i32,
    month_zhi_idx: i32,
    year_gan_idx: i32,
    gender: &str,
    birth_year: i32,
    start_age: f64, // 起运年龄 (由 calculate_start_age 计算)
) -> Vec<serde_json::Value> {
    // 判断阴阳: 阳年(甲丙戊庚壬) vs 阴年(乙丁己辛癸)
    let year_is_yang = year_gan_idx % 2 == 0;

    // 大运顺逆: 阳男阴女顺行，阴男阳女逆行
    let forward = (gender == "male" && year_is_yang) || (gender == "female" && !year_is_yang);

    let mut dayun_cycles = Vec::new();

    for i in 0..10 {
        let cycle_num = if forward { i + 1 } else { -(i + 1) };
        let gan_idx = ((month_gan_idx + cycle_num + 10) % 10 + 10) % 10;
        let zhi_idx = ((month_zhi_idx + cycle_num + 12) % 12 + 12) % 12;

        let start_age_for_cycle = start_age + (i as f64 * 10.0);
        let end_age = start_age_for_cycle + 9.0;

        dayun_cycles.push(json!({
            "cycle": i + 1,
            "gan": TIANGAN[gan_idx as usize],
            "zhi": DIZHI[zhi_idx as usize],
            "gan_wuxing": GAN_WUXING[gan_idx as usize],
            "zhi_wuxing": ZHI_WUXING[zhi_idx as usize],
            "start_age": start_age_for_cycle.round() as i32,
            "end_age": end_age.round() as i32,
            "year_range": format!("{}-{}",
                birth_year + start_age_for_cycle.round() as i32,
                birth_year + end_age.round() as i32
            )
        }));
    }

    dayun_cycles
}

// 计算小运 (Minor Luck)
fn calculate_xiaoyun(
    hour_gan_idx: i32,
    hour_zhi_idx: i32,
    gender: &str,
    birth_year: i32,
    current_year: i32,
) -> serde_json::Value {
    // 小运: 男命从时柱顺推，女命从时柱逆推
    let forward = gender == "male";
    let age = current_year - birth_year;

    let offset = if forward { age } else { -age };
    let gan_idx = ((hour_gan_idx + offset + 10) % 10 + 10) % 10;
    let zhi_idx = ((hour_zhi_idx + offset + 12) % 12 + 12) % 12;

    json!({
        "age": age,
        "year": current_year,
        "gan": TIANGAN[gan_idx as usize],
        "zhi": DIZHI[zhi_idx as usize],
        "gan_wuxing": GAN_WUXING[gan_idx as usize],
        "zhi_wuxing": ZHI_WUXING[zhi_idx as usize]
    })
}

// 计算流年 (Annual Fortune)
fn calculate_liunian(birth_year: i32, current_year: i32, num_years: i32) -> Vec<serde_json::Value> {
    let mut liunian = Vec::new();

    for i in 0..num_years {
        let year = current_year + i;
        let age = year - birth_year;
        let gan_idx = ((year - 4) % 10 + 10) % 10;
        let zhi_idx = ((year - 4) % 12 + 12) % 12;

        liunian.push(json!({
            "year": year,
            "age": age,
            "gan": TIANGAN[gan_idx as usize],
            "zhi": DIZHI[zhi_idx as usize],
            "gan_wuxing": GAN_WUXING[gan_idx as usize],
            "zhi_wuxing": ZHI_WUXING[zhi_idx as usize],
            "zodiac": SHENGXIAO[zhi_idx as usize]
        }));
    }

    liunian
}

// 八字排盘 API
//
// 算法来源：
// - 年柱：以立春(黄经315°)为界换年
// - 月柱：以节气为界换月，五虎遁推月干
// - 日柱：基准日 1970-01-01 = 庚戌，晚子时(23:00后)按次日算
// - 时柱：使用真太阳时确定时辰，五鼠遁推时干
// - 大运：《子平真诠》算法，每3天=1岁
async fn get_bazi(
    Query(params): Query<BaziQuery>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    tracing::info!(
        "🔮 八字排盘请求: datetime={}, gender={:?}, lon={:?}",
        params.datetime,
        params.gender,
        params.lon
    );

    let tz_offset_minutes = params.tz_offset_minutes.unwrap_or(480);
    let fixed_offset = chrono::FixedOffset::east_opt(tz_offset_minutes * 60)
        .unwrap_or_else(|| chrono::FixedOffset::east_opt(8 * 3600).expect("valid UTC+8 offset"));

    // 解析日期时间 - 解析失败直接返回 400，绝不 fallback
    let datetime_utc = if let Ok(dt) = chrono::DateTime::parse_from_rfc3339(&params.datetime) {
        dt.with_timezone(&Utc)
    } else if let Ok(naive) =
        chrono::NaiveDateTime::parse_from_str(&params.datetime, "%Y-%m-%dT%H:%M:%S")
            .or_else(|_| chrono::NaiveDateTime::parse_from_str(&params.datetime, "%Y-%m-%dT%H:%M"))
    {
        if let Some(local_dt) = fixed_offset.from_local_datetime(&naive).single() {
            local_dt.with_timezone(&Utc)
        } else {
            tracing::warn!(
                "❌ 无法按时区解释本地时间: datetime={}, tzOffsetMinutes={}",
                params.datetime,
                tz_offset_minutes
            );
            return Err((
                StatusCode::BAD_REQUEST,
                Json(json!({
                    "error": "无法按时区解释本地时间",
                    "message": format!(
                        "提供的日期时间 '{}' 与时区偏移 '{}' 不兼容，请检查输入格式",
                        params.datetime, tz_offset_minutes
                    )
                })),
            ));
        }
    } else {
        tracing::warn!("❌ 无法解析日期时间: {}", params.datetime);
        return Err((
            StatusCode::BAD_REQUEST,
            Json(json!({
                "error": "无法解析日期时间格式",
                "message": format!("提供的日期时间 '{}' 格式无效，请使用 ISO 8601 格式（如：2025-01-01T12:00:00Z）", params.datetime)
            })),
        ));
    };

    // 获取出生地经度（用于计算真太阳时）
    // 默认使用北京经度 116.4°E
    let longitude = params.lon.unwrap_or(116.4);

    // ==================== 使用 huangji_core 计算四柱 ====================
    // 核心计算使用公共模块，确保星空图、皇极经世、八字排盘数据一致
    let pillars = calc_bazi_pillars(&datetime_utc, longitude);

    let year_gan_idx = pillars.year.0 as i32;
    let year_zhi_idx = pillars.year.1 as i32;
    let month_gan_idx = pillars.month.0 as i32;
    let month_zhi_idx = pillars.month.1 as i32;
    let day_gan_idx = pillars.day.0 as i32;
    let day_zhi_idx = pillars.day.1 as i32;
    let hour_gan_idx = pillars.hour.0 as i32;
    let hour_zhi_idx = pillars.hour.1 as i32;

    let solar_longitude = pillars.solar_longitude;
    // 出生年/当前年按用户传入时区解释，避免 UTC 边界导致年龄偏差
    let birth_year = datetime_utc.with_timezone(&fixed_offset).year();
    let current_year = Utc::now().with_timezone(&fixed_offset).year();

    // 构建四柱（包含十神和藏干）
    let create_pillar = |gan_idx: i32, zhi_idx: i32, day_gan_idx: usize| -> serde_json::Value {
        let gi = gan_idx as usize % 10;
        let zi = zhi_idx as usize % 12;
        let nayin_idx = ((gi / 2) * 6 + zi / 2) % 30;

        // 计算天干十神
        let gan_ten_god = calculate_ten_god(day_gan_idx, gi);

        // 计算地支藏干及其十神
        let hidden_stems = get_hidden_stems_with_gods(zi, day_gan_idx);

        json!({
            "gan": TIANGAN[gi],
            "zhi": DIZHI[zi],
            "gan_wuxing": GAN_WUXING[gi],
            "zhi_wuxing": ZHI_WUXING[zi],
            "zhi_animal": SHENGXIAO[zi],
            "nayin": NAYIN[nayin_idx],
            "gan_ten_god": gan_ten_god,
            "hidden_stems": hidden_stems
        })
    };

    let day_gan_idx_usize = day_gan_idx as usize % 10;
    let year_pillar = create_pillar(year_gan_idx, year_zhi_idx, day_gan_idx_usize);
    let month_pillar = create_pillar(month_gan_idx, month_zhi_idx, day_gan_idx_usize);
    let day_pillar = create_pillar(day_gan_idx, day_zhi_idx, day_gan_idx_usize);
    let hour_pillar = create_pillar(hour_gan_idx, hour_zhi_idx, day_gan_idx_usize);

    // 统计五行
    let mut wuxing_counts: std::collections::HashMap<String, i32> =
        std::collections::HashMap::new();
    wuxing_counts.insert("木".to_string(), 0);
    wuxing_counts.insert("火".to_string(), 0);
    wuxing_counts.insert("土".to_string(), 0);
    wuxing_counts.insert("金".to_string(), 0);
    wuxing_counts.insert("水".to_string(), 0);

    // 统计天干五行
    for idx in [year_gan_idx, month_gan_idx, day_gan_idx, hour_gan_idx] {
        let wx = GAN_WUXING[idx as usize % 10]
            .replace("阳", "")
            .replace("阴", "");
        *wuxing_counts.entry(wx).or_insert(0) += 1;
    }
    // 统计地支五行
    for idx in [year_zhi_idx, month_zhi_idx, day_zhi_idx, hour_zhi_idx] {
        let wx = ZHI_WUXING[idx as usize % 12]
            .replace("阳", "")
            .replace("阴", "");
        *wuxing_counts.entry(wx).or_insert(0) += 1;
    }

    // 日主分析
    let day_master = GAN_WUXING[day_gan_idx as usize % 10];
    let day_master_wx = day_master.replace("阳", "").replace("阴", "");
    let day_master_count = wuxing_counts.get(&day_master_wx).unwrap_or(&0);

    let strength = if *day_master_count >= 3 {
        "strong"
    } else if *day_master_count <= 1 {
        "weak"
    } else {
        "balanced"
    };

    // 缺失的五行
    let missing: Vec<&str> = ["木", "火", "土", "金", "水"]
        .iter()
        .filter(|wx| *wuxing_counts.get(**wx).unwrap_or(&0) == 0)
        .copied()
        .collect();

    let gender = params.gender.unwrap_or_else(|| "male".to_string());

    // 计算起运年龄 (使用 huangji_core 精确计算)
    // 《子平真诠》算法：出生日到节气的天数 / 3 = 起运岁数
    let jd = huangji_core::astro::solar::utc_to_jd(&datetime_utc);
    let is_male = gender == "male";
    let start_age = calc_dayun_start_age(jd, pillars.year.0, is_male);

    // 计算大运
    let dayun = calculate_dayun(
        month_gan_idx,
        month_zhi_idx,
        year_gan_idx,
        &gender,
        birth_year, // 出生年份 (按用户时区)
        start_age,
    );

    // 计算当前小运
    let xiaoyun = calculate_xiaoyun(
        hour_gan_idx,
        hour_zhi_idx,
        &gender,
        birth_year,
        current_year,
    );

    // 计算流年 (当前年+未来5年)
    let liunian = calculate_liunian(birth_year, current_year, 6);

    // 日主十神分析
    let day_gan_str = TIANGAN[day_gan_idx as usize % 10];

    // 获取当前节气 (使用 huangji_core)
    let current_solar_term = pillars.solar_term.name();

    // 计算真太阳时 (用于调试/验证)
    let tst_hour = true_solar_hour(&datetime_utc, longitude);

    Ok(Json(json!({
        "year_pillar": year_pillar,
        "month_pillar": month_pillar,
        "day_pillar": day_pillar,
        "hour_pillar": hour_pillar,
        "wuxing_analysis": {
            "day_master": day_master,
            "day_master_gan": day_gan_str,
            "day_master_strength": strength,
            "wuxing_counts": wuxing_counts,
            "missing_wuxing": missing
        },
        "ten_gods_summary": {
            "year_gan": year_pillar["gan_ten_god"],
            "month_gan": month_pillar["gan_ten_god"],
            "day_gan": day_pillar["gan_ten_god"],
            "hour_gan": hour_pillar["gan_ten_god"]
        },
        "dayun": dayun,
        "xiaoyun": xiaoyun,
        "liunian": liunian,
        "gender": gender,
        "birth_year": birth_year,
        "zodiac": SHENGXIAO[year_zhi_idx as usize % 12],
        "solar_term": current_solar_term,
        "start_age": start_age.round() as i32,
        "solar_longitude": solar_longitude,
        "true_solar_hour": tst_hour,
        "is_late_zi": pillars.is_late_zi,
        "longitude": longitude,
        "tz_offset_minutes": tz_offset_minutes
    })))
}

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
    use super::{extract_events_array, index_events_by_year};
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
}
