use axum::{
    routing::{get, post},
    Json, Router, extract::{Query, Path},
};
use axum::response::IntoResponse;
use axum::http::HeaderValue;
use chrono::{DateTime, Utc, Datelike};
use serde::{Deserialize, Serialize};
use tower_http::cors::CorsLayer;
use huangji_core::{fortune, sky, data, algorithm};
use once_cell::sync::Lazy;
use std::sync::RwLock;
use std::collections::HashMap;
use std::fs::File;
use std::io::BufReader;
use std::io::Write;
use std::env;
use reqwest::Client;
use tokio::task;
use sha2::{Sha256, Digest};
use serde_json::json;

#[tokio::main]
async fn main() {
    // 初始化 tracing
    tracing_subscriber::fmt::init();

    // 加载数据
    let current_dir = env::current_dir().unwrap();
    // 假设 backend 运行在 workspace root 或 backend 目录
    // 我们需要找到 huangji_core/data/year_mapping.json
    let data_path = current_dir.join("huangji_core/data/year_mapping.json");
    
    println!("Loading data from: {:?}", data_path);
    if let Err(e) = data::load_data(data_path.to_str().unwrap()) {
        eprintln!("Failed to load data: {}. Trying absolute path...", e);
        // Fallback to hardcoded absolute path for this environment
        let abs_path = "/Users/bill/Desktop/hjjs/huangji-jingshi-web/huangji_core/data/year_mapping.json";
        if let Err(e2) = data::load_data(abs_path) {
            eprintln!("Failed to load data from absolute path: {}", e2);
        }
    }

    // 加载历史事件数据
    let hist_path_default = current_dir.join("backend/data/history.json");
    if let Err(e) = load_history_data(hist_path_default.to_str().unwrap_or_default()) {
        eprintln!("Failed to load history from default: {}", e);
        let abs_hist = "/Users/bill/Desktop/hjjs/huangji-jingshi-web/backend/data/history.json";
        let _ = load_history_data(abs_hist);
    }

    // 加载天文数据哈希清单
    init_celestial_hashes();

    // 允许 CORS
    let cors = CorsLayer::permissive();

    // 构建路由
    let app = Router::new()
        .route("/health", get(health_check))
        .route("/api/fortune", get(get_fortune))
        .route("/api/sky", get(get_sky))
        .route("/api/sky-and-fortune", get(get_sky_and_fortune))
        .route("/api/timezone", get(get_timezone))
        .route("/api/timeline", get(get_timeline))
        .route("/api/history", get(get_history))
        .route("/api/history/related", get(get_related_history))
        .route("/api/celestial/data/*path", get(celestial_data))
        .route("/api/celestial/cache/index", get(get_cache_index))
        .route("/api/celestial/cache/preload", post(preload_cache))
        .route("/api/celestial/cache/clear", post(clear_cache))
        .route("/api/settings/sky", get(get_sky_settings).post(update_sky_settings))
        .route("/api/history/import-excel", get(import_history_excel))
        .route("/api/mapping/import-excel", get(import_mapping_excel))
        .route("/api/mapping/import-json", get(import_mapping_json))
        .route("/api/mapping/get", get(get_mapping_by_year))
        .route("/api/excel/inspect", get(inspect_excel))
        .layer(cors);

    // 运行服务
    let listener = tokio::net::TcpListener::bind("0.0.0.0:8080").await.unwrap();
    println!("Server running on http://0.0.0.0:8080");
    axum::serve(listener, app).await.unwrap();
}

async fn health_check() -> &'static str {
    "OK"
}

#[derive(Deserialize)]
struct ApiParams {
    datetime: Option<String>, // ISO 8601 string
    lat: Option<f64>,
    lon: Option<f64>,
    delta_t_provider: Option<String>,
    accuracy: Option<String>,
}

async fn get_fortune(Query(params): Query<ApiParams>) -> Json<fortune::FortuneResponse> {
    let dt = parse_datetime(params.datetime);
    let req = fortune::FortuneRequest { datetime: dt };
    Json(fortune::compute_fortune(&req))
}

async fn get_sky(Query(params): Query<ApiParams>) -> Json<sky::SkyResponse> {
    let dt = parse_datetime(params.datetime);
    let default_provider = env::var("DELTA_T_PROVIDER_DEFAULT").ok();
    let default_accuracy = env::var("ASTRO_ACCURACY_DEFAULT").ok();
    let req = sky::SkyRequest {
        datetime: dt,
        lat_deg: params.lat.unwrap_or(39.9), // 默认北京
        lon_deg: params.lon.unwrap_or(116.4),
        delta_t_provider: params.delta_t_provider.or(default_provider),
        accuracy: params.accuracy.or(default_accuracy),
    };
    Json(sky::compute_sky(&req))
}

#[derive(Serialize)]
struct CombinedResponse {
    fortune: fortune::FortuneResponse,
    sky: sky::SkyResponse,
}

async fn get_sky_and_fortune(Query(params): Query<ApiParams>) -> Json<CombinedResponse> {
    let dt = parse_datetime(params.datetime);
    let lat = params.lat.unwrap_or(39.9);
    let lon = params.lon.unwrap_or(116.4);
    let default_provider = env::var("DELTA_T_PROVIDER_DEFAULT").ok();
    let default_accuracy = env::var("ASTRO_ACCURACY_DEFAULT").ok();

    let fortune_req = fortune::FortuneRequest { datetime: dt };
    let sky_req = sky::SkyRequest {
        datetime: dt,
        lat_deg: lat,
        lon_deg: lon,
        delta_t_provider: params.delta_t_provider.or(default_provider),
        accuracy: params.accuracy.or(default_accuracy),
    };

    Json(CombinedResponse {
        fortune: fortune::compute_fortune(&fortune_req),
        sky: sky::compute_sky(&sky_req),
    })
}

async fn get_timeline(Query(params): Query<ApiParams>) -> Json<algorithm::TimelineData> {
    let dt = parse_datetime(params.datetime);
    let year = dt.year();
    Json(algorithm::get_timeline_info(year))
}

fn parse_datetime(dt_str: Option<String>) -> DateTime<Utc> {
    match dt_str {
        Some(s) => s.parse::<DateTime<Utc>>().unwrap_or(Utc::now()),
        None => Utc::now(),
    }
}

#[derive(Serialize)]
struct TimezoneResponse {
    zone_name: Option<String>,
    offset_seconds: i32,
    source: String,
}

#[allow(dead_code)]
#[derive(Deserialize)]
struct TzdbResp {
    status: Option<String>,
    #[serde(rename = "zoneName")] zone_name: Option<String>,
    #[serde(rename = "gmtOffset")] gmt_offset: Option<i32>,
}

async fn get_timezone(Query(params): Query<ApiParams>) -> axum::response::Response {
    let dt = parse_datetime(params.datetime);
    let lat = params.lat.unwrap_or(39.9);
    let lon = params.lon.unwrap_or(116.4);
    let day_key = dt.date_naive().to_string();
    let lat_s = format!("{:.4}", lat);
    let lon_s = format!("{:.4}", lon);
    let cache_key = format!("{}|{}|{}", lat_s, lon_s, day_key);
    {
        let store = TIMEZONE_CACHE.read().unwrap();
        if let Some(entry) = store.get(&cache_key) {
            if entry.expires_at > Utc::now().timestamp() {
                let resp = TimezoneResponse { zone_name: entry.zone_name.clone(), offset_seconds: entry.offset_seconds, source: "cache".to_string() };
                return Json(resp).into_response();
            }
        }
    }
    if let Ok(key) = env::var("TIMEZONEDB_KEY") {
        let client = Client::new();
        let time = dt.timestamp();
        let url = format!(
            "https://api.timezonedb.com/v2.1/get-time-zone?key={}&format=json&by=position&lat={}&lng={}&time={}",
            key, lat, lon, time
        );
        match client.get(url).send().await {
            Ok(resp) => {
                if let Ok(json) = resp.json::<TzdbResp>().await {
                    if let Some(offset) = json.gmt_offset {
                        let zone = json.zone_name;
                        let expires = Utc::now().timestamp() + 86400;
                        {
                            let mut store = TIMEZONE_CACHE.write().unwrap();
                            store.insert(cache_key.clone(), TzCacheEntry { zone_name: zone.clone(), offset_seconds: offset, expires_at: expires });
                        }
                        let tz = TimezoneResponse { zone_name: zone, offset_seconds: offset, source: "timezonedb".to_string() };
                        return Json(tz).into_response();
                    }
                }
                return axum::response::Response::builder()
                    .status(502)
                    .body(axum::body::Body::from("Bad Gateway: invalid timezone response"))
                    .unwrap();
            }
            Err(_) => {
                let mut guess = (lon / 15.0).round() as i32;
                if guess < -12 { guess = -12; }
                if guess > 14 { guess = 14; }
                let offset = guess * 3600;
                let expires = Utc::now().timestamp() + 86400;
                {
                    let mut store = TIMEZONE_CACHE.write().unwrap();
                    store.insert(cache_key.clone(), TzCacheEntry { zone_name: None, offset_seconds: offset, expires_at: expires });
                }
                let tz = TimezoneResponse { zone_name: None, offset_seconds: offset, source: "approx".to_string() };
                return Json(tz).into_response();
            }
        }
    }
    let mut guess = (lon / 15.0).round() as i32;
    if guess < -12 { guess = -12; }
    if guess > 14 { guess = 14; }
    let offset = guess * 3600;
    let expires = Utc::now().timestamp() + 86400;
    {
        let mut store = TIMEZONE_CACHE.write().unwrap();
        store.insert(cache_key.clone(), TzCacheEntry { zone_name: None, offset_seconds: offset, expires_at: expires });
    }
    let tz = TimezoneResponse { zone_name: None, offset_seconds: offset, source: "approx".to_string() };
    Json(tz).into_response()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct HistoryEvent {
    year: i32,
    title: String,
    description: String,
    category: Option<String>,
    tags: Option<Vec<String>>, // e.g., ["战争", "科技"]
}

#[derive(Deserialize)]
struct HistoryQuery {
    start: Option<i32>,
    end: Option<i32>,
    category: Option<String>,
    page: Option<usize>,
    page_size: Option<usize>,
    q: Option<String>,
    tags: Option<String>,
    tags_any: Option<String>,
}

static HISTORY_STORE: Lazy<RwLock<Vec<HistoryEvent>>> = Lazy::new(|| RwLock::new(Vec::new()));
#[derive(Clone, Serialize, Deserialize)]
struct SkySettings { show_const: bool, show_xiu: bool, zh_planet_names: bool, culture: String }
static SKY_SETTINGS: Lazy<RwLock<SkySettings>> = Lazy::new(|| RwLock::new(SkySettings { show_const: true, show_xiu: true, zh_planet_names: true, culture: "hj".to_string() }));
#[derive(Clone)]
struct TzCacheEntry {
    zone_name: Option<String>,
    offset_seconds: i32,
    expires_at: i64,
}
static TIMEZONE_CACHE: Lazy<RwLock<HashMap<String, TzCacheEntry>>> = Lazy::new(|| RwLock::new(HashMap::new()));

static CELESTIAL_HASH_CN: &str = "d5558cd419c8d46bdc958064cb97f963d1ea793866414c025906ec15033512ed";
static CELESTIAL_HASH_STARS_6: &str = "0297b8fa3adfbce1dc26566f61c4abcc1df4f29c6a28729ca06b56d1c6d25602";
static CELESTIAL_HASH_CONSTELLATIONS: &str = "ab4ae692027cbc042c0d6791a84456a65eb7c55656107fd00c58ff6e55d4d8b2";
static CELESTIAL_HASH_CONSTELLATIONS_LINES: &str = "294f66bef5d5cf50b1e17f16d2efa1d97a15131612c68dd935adef6e7373e13c";
static CELESTIAL_HASH_CONSTELLATIONS_BOUNDS: &str = "f2e2687af6b20b24567879f838c21874d412efcc93ecc1966be07e78431cc196";
static CELESTIAL_HASH_PLANETS: &str = "5fca7ea95880f6feeaab75f306a058aa36f86deedd45ec82cd37e48d20899953";
static CELESTIAL_HASH_MW: &str = "aee221a7a0e879418e685de00c3e68fbdfac5667c0a8aab74929ef9cf4aab4fb";
static CELESTIAL_HASHES: Lazy<RwLock<HashMap<String, String>>> = Lazy::new(|| RwLock::new(HashMap::new()));

fn load_history_data(path: &str) -> anyhow::Result<()> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);
    #[derive(Deserialize)]
    struct Wrapper { events: Vec<HistoryEvent> }
    let wrapper: Wrapper = serde_json::from_reader(reader)?;
    let mut store = HISTORY_STORE.write().unwrap();
    *store = wrapper.events;
    println!("Loaded {} history events.", store.len());
    Ok(())
}

#[axum::debug_handler]
async fn celestial_data(Path(path): Path<String>) -> axum::response::Response {
    use axum::http::StatusCode;
    let clean = path.trim_start_matches('/');
    let allowed = [
        "cultures/cn.json",
        "cultures/hj.json",
        "stars.6.json",
        "mw.json",
        "constellations.json",
        "constellations.lines.json",
        "constellations.bounds.json",
        "planets.json",
        "constellations.cn.json",
        "constellations.lines.cn.json",
        "constellations.bounds.cn.json",
        "starnames.cn.json",
        "dsonames.cn.json",
        "planets.cn.json",
    ];
    if !allowed.contains(&clean) && !clean.starts_with("cultures/") {
        return axum::response::Response::builder()
            .status(StatusCode::BAD_REQUEST)
            .body(axum::body::Body::from("Unsupported path"))
            .unwrap();
    }

    let cache_root = "backend/data/celestial";
    let cache_path = format!("{}/{}", cache_root, clean);
    // 1) Serve from backend cache if present
    if let Ok(mut f) = File::open(&cache_path) {
        use std::io::Read;
        let mut buf = Vec::new();
        if f.read_to_end(&mut buf).is_ok() {
            if verify_hash(&clean, &buf) {
                // try to read index entry for meta
                let entry = get_index_entry(&clean);
                let mut hasher = Sha256::new();
                hasher.update(&buf);
                let got = format!("{:x}", hasher.finalize());
                let fallback_etag = format!("W/\"sha256:{}\"", got);
                let etag = entry.as_ref().and_then(|e| e.etag.clone()).or(Some(fallback_etag));
                let last_mod = entry.as_ref().and_then(|e| e.last_modified.clone());
                return ok_json_meta(buf, etag, last_mod);
            }
        }
    }

    // 2) Dynamic CN generators with guaranteed availability
    if clean == "constellations.cn.json" { return ok_json(gen_cn_constellations_points()); }
    if clean == "constellations.lines.cn.json" { return ok_json(gen_cn_constellations_lines()); }
    if clean == "constellations.bounds.cn.json" { return ok_json(gen_cn_constellations_bounds()); }
    if clean == "starnames.cn.json" { return ok_json(gen_cn_starnames()); }
    if clean == "dsonames.cn.json" { return ok_json(gen_cn_dsonames()); }

    // 3) Try local frontend public data
    let local_candidates = [
        // Dev/public paths
        format!("frontend/public/data/{}", clean),
        format!("huangji-jingshi-web/frontend/public/data/{}", clean),
        format!("./frontend/public/data/{}", clean),
        // Built dist paths
        format!("frontend/dist/data/{}", clean),
        format!("huangji-jingshi-web/frontend/dist/data/{}", clean),
        format!("./frontend/dist/data/{}", clean),
    ];
    for p in &local_candidates {
        if let Ok(mut f) = File::open(p) {
            use std::io::Read;
            let mut buf = Vec::new();
            if f.read_to_end(&mut buf).is_ok() {
                if verify_hash(&clean, &buf) {
                    let mut hasher = Sha256::new();
                    hasher.update(&buf);
                    let got = format!("{:x}", hasher.finalize());
                    let etag = Some(format!("W/\"sha256:{}\"", got));
                    return ok_json_meta(buf, etag, None);
                }
            }
        }
    }

    // 4) Remote fetch (if available)

    let client = Client::new();
    let mut roots = get_env_roots();
    if roots.is_empty() {
        roots = vec![
            "https://raw.githubusercontent.com/ofrohn/celestial/master/data/".to_string(),
            "https://cdn.jsdelivr.net/gh/ofrohn/celestial@master/data/".to_string(),
            "https://fastly.jsdelivr.net/gh/ofrohn/celestial@master/data/".to_string(),
            "https://ofrohn.github.io/data/".to_string(),
        ];
    }
    for r in roots.iter() {
        let url = format!("{}{}", r, clean);
        if let Ok(resp) = client.get(url).send().await {
            if resp.status().is_success() {
                let headers_clone = resp.headers().clone();
                if let Ok(bytes) = resp.bytes().await {
                    let buf = bytes.to_vec();
                    if !verify_hash(&clean, &buf) { continue; }
                    let mut hasher = Sha256::new();
                    hasher.update(&buf);
                    let got = format!("{:x}", hasher.finalize());
                    // write-through cache
                    if let Some(parent) = std::path::Path::new(&cache_path).parent() {
                        let _ = std::fs::create_dir_all(parent);
                    }
                    let _ = std::fs::write(&cache_path, &buf);
                    let etag_hdr = headers_clone.get("etag").and_then(|h| h.to_str().ok()).map(|s| s.to_string());
                    let last_mod_hdr = headers_clone.get("last-modified").and_then(|h| h.to_str().ok()).map(|s| s.to_string());
                    update_cache_index(&clean, &got, r, buf.len(), etag_hdr.clone(), last_mod_hdr.clone());
                    let etag = etag_hdr.or(Some(format!("W/\"sha256:{}\"", got)));
                    return ok_json_meta(buf, etag, last_mod_hdr);
                }
            }
        }
    }
    axum::response::Response::builder()
        .status(StatusCode::BAD_GATEWAY)
        .body(axum::body::Body::from("Bad Gateway: celestial data unavailable"))
        .unwrap()
}

fn ok_json_meta(buf: Vec<u8>, etag: Option<String>, last_modified: Option<String>) -> axum::response::Response {
    use axum::http::header;
    let mut builder = axum::response::Response::builder()
        .status(axum::http::StatusCode::OK)
        .header(header::CONTENT_TYPE, "application/json");
    if let Some(e) = etag { if let Ok(hv) = HeaderValue::from_str(&e) { builder = builder.header(header::ETAG, hv); } }
    if let Some(lm) = last_modified { if let Ok(hv) = HeaderValue::from_str(&lm) { builder = builder.header(header::LAST_MODIFIED, hv); } }
    builder.body(axum::body::Body::from(buf)).unwrap()
}

#[allow(dead_code)]
fn ok_json(buf: Vec<u8>) -> axum::response::Response { ok_json_meta(buf, None, None) }

fn verify_hash(path: &str, buf: &[u8]) -> bool {
    fn expected_hash(path: &str) -> Option<String> {
        let store = CELESTIAL_HASHES.read().unwrap();
        if let Some(v) = store.get(path) { return Some(v.clone()); }
        if path.starts_with("cultures/") { return None; }
        match path {
            "stars.6.json" => Some(CELESTIAL_HASH_STARS_6.to_string()),
            "constellations.json" => Some(CELESTIAL_HASH_CONSTELLATIONS.to_string()),
            "constellations.lines.json" => Some(CELESTIAL_HASH_CONSTELLATIONS_LINES.to_string()),
            "constellations.bounds.json" => Some(CELESTIAL_HASH_CONSTELLATIONS_BOUNDS.to_string()),
            "planets.json" => Some(CELESTIAL_HASH_PLANETS.to_string()),
            "mw.json" => Some(CELESTIAL_HASH_MW.to_string()),
            _ => None,
        }
    }
    if let Some(exp) = expected_hash(path) {
        let mut hasher = Sha256::new();
        hasher.update(buf);
        let got = format!("{:x}", hasher.finalize());
        got == exp
    } else { true }
}

fn ecl_to_eq(lambda_deg: f64, beta_deg: f64) -> (f64, f64) {
    let eps = 23.43929111f64.to_radians();
    let lam = lambda_deg.to_radians();
    let beta = beta_deg.to_radians();
    let y = lam.sin() * eps.cos() - beta.tan() * eps.sin();
    let x = lam.cos();
    let alpha = y.atan2(x);
    let delta = (beta.sin() * eps.cos() + beta.cos() * eps.sin() * lam.sin()).asin();
    let mut ra = alpha.to_degrees();
    if ra > 180.0 { ra -= 360.0; }
    if ra < -180.0 { ra += 360.0; }
    let dec = delta.to_degrees();
    (ra, dec)
}

fn gen_cn_constellations_points() -> Vec<u8> {
    let mansions = vec![
        "角","亢","氐","房","心","尾","箕","斗",
        "牛","女","虚","危","室","壁","奎","娄",
        "胃","昴","毕","觜","参","井","鬼","柳",
        "星","张","翼","轸"
    ];
    let seg = 360.0 / mansions.len() as f64;
    let mut features = Vec::new();
    for (i, name) in mansions.iter().enumerate() {
        let lam = (i as f64 + 0.5) * seg;
        let (ra, dec) = ecl_to_eq(lam, 0.0);
        let feat = json!({
            "type": "Feature",
            "id": name,
            "properties": { "name": name, "desig": name, "rank": i + 1 },
            "geometry": { "type": "Point", "coordinates": [ra, dec] }
        });
        features.push(feat);
    }
    // 三垣中心点（近似）
    let sanyuan = vec![
        ("紫微垣", 0.0, 75.0),
        ("太微垣", 195.0, 22.0),
        ("天市垣", 270.0, 30.0),
    ];
    for (name, ra, dec) in sanyuan.into_iter() {
        let feat = json!({
            "type": "Feature",
            "id": name,
            "properties": { "name": name, "desig": name },
            "geometry": { "type": "Point", "coordinates": [adjust_ra(ra), dec] }
        });
        features.push(feat);
    }
    json!({ "type": "FeatureCollection", "features": features }).to_string().into_bytes()
}

fn gen_cn_constellations_lines() -> Vec<u8> {
    let n = 28usize;
    let seg = 360.0 / n as f64;
    let mut features = Vec::new();
    for i in 0..n {
        let lam = i as f64 * seg;
        let (ra1, dec1) = ecl_to_eq(lam, -1.0);
        let (ra2, dec2) = ecl_to_eq(lam, 1.0);
        let name = match i {
            0 => "角", 1 => "亢", 2 => "氐", 3 => "房", 4 => "心", 5 => "尾", 6 => "箕", 7 => "斗",
            8 => "牛", 9 => "女", 10 => "虚", 11 => "危", 12 => "室", 13 => "壁", 14 => "奎", 15 => "娄",
            16 => "胃", 17 => "昴", 18 => "毕", 19 => "觜", 20 => "参", 21 => "井", 22 => "鬼", 23 => "柳",
            24 => "星", 25 => "张", 26 => "翼", _ => "轸"
        };
        let feat = json!({
            "type": "Feature",
            "id": name,
            "geometry": { "type": "MultiLineString", "coordinates": [[[ra1, dec1], [ra2, dec2]]] }
        });
        features.push(feat);
    }
    json!({ "type": "FeatureCollection", "features": features }).to_string().into_bytes()
}

fn gen_cn_constellations_bounds() -> Vec<u8> {
    let n = 28usize;
    let seg = 360.0 / n as f64;
    let mut features = Vec::new();
    for i in 0..n {
        let lam1 = i as f64 * seg;
        let lam2 = (i as f64 + 1.0) * seg;
        let mut coords = Vec::new();
        for k in 0..=8 { let t = k as f64 / 8.0; let lam = lam1 + (lam2 - lam1) * t; let (ra, dec) = ecl_to_eq(lam, 1.0); coords.push(vec![ra, dec]); }
        for k in (0..=8).rev() { let t = k as f64 / 8.0; let lam = lam1 + (lam2 - lam1) * t; let (ra, dec) = ecl_to_eq(lam, -1.0); coords.push(vec![ra, dec]); }
        if let Some(first) = coords.first().cloned() { coords.push(first); }
        let name = match i {
            0 => "角", 1 => "亢", 2 => "氐", 3 => "房", 4 => "心", 5 => "尾", 6 => "箕", 7 => "斗",
            8 => "牛", 9 => "女", 10 => "虚", 11 => "危", 12 => "室", 13 => "壁", 14 => "奎", 15 => "娄",
            16 => "胃", 17 => "昴", 18 => "毕", 19 => "觜", 20 => "参", 21 => "井", 22 => "鬼", 23 => "柳",
            24 => "星", 25 => "张", 26 => "翼", _ => "轸"
        };
        let feat = json!({
            "type": "Feature",
            "id": name,
            "geometry": { "type": "Polygon", "coordinates": [coords] }
        });
        features.push(feat);
    }
    // 三垣近似边界（紫微：极区环；太微/天市：矩形）
    // 紫微垣：dec=70°环
    let mut circle: Vec<Vec<f64>> = Vec::new();
    for k in 0..=36 { let t = k as f64 / 36.0; let ra = adjust_ra(t * 360.0); circle.push(vec![ra, 70.0]); }
    features.push(json!({ "type": "Feature", "id": "紫微垣", "geometry": { "type": "Polygon", "coordinates": [circle] } }));
    // 太微垣：RA 150..220, Dec 10..35
    let rect_tw = vec![ vec![adjust_ra(150.0), 10.0], vec![adjust_ra(220.0), 10.0], vec![adjust_ra(220.0), 35.0], vec![adjust_ra(150.0), 35.0], vec![adjust_ra(150.0), 10.0] ];
    features.push(json!({ "type": "Feature", "id": "太微垣", "geometry": { "type": "Polygon", "coordinates": [rect_tw] } }));
    // 天市垣：RA 240..300, Dec 15..45
    let rect_ts = vec![ vec![adjust_ra(240.0), 15.0], vec![adjust_ra(300.0), 15.0], vec![adjust_ra(300.0), 45.0], vec![adjust_ra(240.0), 45.0], vec![adjust_ra(240.0), 15.0] ];
    features.push(json!({ "type": "Feature", "id": "天市垣", "geometry": { "type": "Polygon", "coordinates": [rect_ts] } }));
    json!({ "type": "FeatureCollection", "features": features }).to_string().into_bytes()
}

fn adjust_ra(ra_deg: f64) -> f64 { if ra_deg > 180.0 { ra_deg - 360.0 } else { ra_deg } }

fn gen_cn_starnames() -> Vec<u8> {
    // 选取皇极体系常用亮星与星官代表：北斗七星、织女/牛郎、参宿七、角宿一、心宿二、北极星、南斗六星（近似）
    let mut features = Vec::new();
    let add = |features: &mut Vec<serde_json::Value>, name: &str, ra_deg: f64, dec_deg: f64| {
        let feat = json!({
            "type": "Feature",
            "id": name,
            "properties": { "name": name },
            "geometry": { "type": "Point", "coordinates": [adjust_ra(ra_deg), dec_deg] }
        });
        features.push(feat);
    };
    // 北斗七星（Dubhe, Merak, Phecda, Megrez, Alioth, Mizar, Alkaid）
    add(&mut features, "天枢", 165.75, 61.75);
    add(&mut features, "天璇", 165.5, 56.4);
    add(&mut features, "天玑", 178.2, 53.7);
    add(&mut features, "天权", 183.9, 57.0);
    add(&mut features, "玉衡", 193.5, 55.9);
    add(&mut features, "开阳", 201.3, 54.9);
    add(&mut features, "摇光", 210.0, 49.3);
    // 织女 / 牛郎
    add(&mut features, "织女", 279.2, 38.8);
    add(&mut features, "牵牛", 297.7, 8.9);
    // 参宿七（Betelgeuse），角宿一（Spica），心宿二（Antares），北极星（Polaris）
    add(&mut features, "参宿七", 88.8, 7.4);
    add(&mut features, "角宿一", 201.9, -11.2);
    add(&mut features, "心宿二", 247.2, -26.4);
    add(&mut features, "北极星", 37.9, 89.3);
    // 南斗六星（近似）
    add(&mut features, "南斗一", 276.0, -30.0);
    add(&mut features, "南斗二", 278.0, -31.0);
    add(&mut features, "南斗三", 280.0, -32.0);
    add(&mut features, "南斗四", 282.0, -33.0);
    add(&mut features, "南斗五", 284.0, -34.0);
    add(&mut features, "南斗六", 286.0, -35.0);
    json!({ "type": "FeatureCollection", "features": features }).to_string().into_bytes()
}

fn gen_cn_dsonames() -> Vec<u8> { json!({ "type": "FeatureCollection", "features": [] }).to_string().into_bytes() }

fn get_env_roots() -> Vec<String> {
    let raw = env::var("CELESTIAL_DATA_ROOTS").unwrap_or_default();
    raw.split(|c| c == ',' || c == ';' || c == '|')
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect()
}

#[derive(Serialize, Deserialize, Clone)]
struct CacheEntry { path: String, sha256: String, source: String, fetched_at: i64, size: usize, etag: Option<String>, last_modified: Option<String> }
#[derive(Serialize, Deserialize, Clone)]
struct CacheIndex { files: Vec<CacheEntry> }

fn update_cache_index(path: &str, sha256: &str, source: &str, size: usize, etag: Option<String>, last_modified: Option<String>) {
    let idx_path = "backend/data/celestial/index.json";
    let mut current: CacheIndex = std::fs::read(idx_path)
        .ok()
        .and_then(|b| serde_json::from_slice::<CacheIndex>(&b).ok())
        .unwrap_or(CacheIndex { files: Vec::new() });
    let ts = Utc::now().timestamp();
    let mut replaced = false;
    for e in current.files.iter_mut() {
        if e.path == path {
            e.sha256 = sha256.to_string();
            e.source = source.to_string();
            e.fetched_at = ts;
            e.size = size;
            e.etag = etag.clone();
            e.last_modified = last_modified.clone();
            replaced = true;
            break;
        }
    }
    if !replaced {
        current.files.push(CacheEntry { path: path.to_string(), sha256: sha256.to_string(), source: source.to_string(), fetched_at: ts, size, etag, last_modified });
    }
    if let Some(parent) = std::path::Path::new(idx_path).parent() { let _ = std::fs::create_dir_all(parent); }
    let _ = std::fs::write(idx_path, serde_json::to_vec(&current).unwrap_or_default());
}

#[axum::debug_handler]
async fn get_cache_index() -> Json<CacheIndex> {
    let idx_path = "backend/data/celestial/index.json";
    let current: CacheIndex = std::fs::read(idx_path)
        .ok()
        .and_then(|b| serde_json::from_slice::<CacheIndex>(&b).ok())
        .unwrap_or(CacheIndex { files: Vec::new() });
    Json(current)
}

#[axum::debug_handler]
async fn clear_cache() -> Json<bool> {
    let root = std::path::Path::new("backend/data/celestial");
    if root.exists() {
        let _ = std::fs::remove_dir_all(root);
    }
    let _ = std::fs::create_dir_all(root);
    let _ = std::fs::write("backend/data/celestial/index.json", serde_json::to_vec(&CacheIndex { files: Vec::new() }).unwrap_or_default());
    Json(true)
}

#[axum::debug_handler]
async fn get_sky_settings() -> Json<SkySettings> {
    let s = SKY_SETTINGS.read().unwrap().clone();
    Json(s)
}

#[axum::debug_handler]
async fn update_sky_settings(Json(payload): Json<SkySettings>) -> Json<SkySettings> {
    let mut s = SKY_SETTINGS.write().unwrap();
    *s = payload.clone();
    Json(payload)
}

fn get_index_entry(path: &str) -> Option<CacheEntry> {
    let idx_path = "backend/data/celestial/index.json";
    std::fs::read(idx_path)
        .ok()
        .and_then(|b| serde_json::from_slice::<CacheIndex>(&b).ok())
        .and_then(|idx| idx.files.into_iter().find(|e| e.path == path))
}

async fn get_history(Query(q): Query<HistoryQuery>) -> axum::response::Response {
    let start = q.start.unwrap_or(i32::MIN);
    let end = q.end.unwrap_or(i32::MAX);
    let category = q.category.as_deref();
    let page = q.page.unwrap_or(1);
    let page_size = q.page_size.unwrap_or(50);
    let qtext = q.q.as_deref();

    let supabase_url = env::var("SUPABASE_URL").ok();
    let supabase_key = env::var("SUPABASE_KEY").ok();
    if let (Some(url), Some(key)) = (supabase_url, supabase_key) {
        let client = Client::new();
        let mut req = client
            .get(format!("{}/rest/v1/history_events", url))
            .header("apikey", &key)
            .header("Authorization", format!("Bearer {}", key))
            .header("Accept", "application/json")
            .header("Prefer", "count=exact")
            .query(&[("select", "*"), ("order", "year.asc")]);
        req = req.query(&[("year", format!("gte.{}", start)), ("year", format!("lte.{}", end))]);
        if let Some(c) = category { req = req.query(&[("category", format!("eq.{}", c))]); }
        let offset = (page.saturating_sub(1)) * page_size;
        req = req.query(&[("limit", page_size.to_string()), ("offset", offset.to_string())]);
        if let Some(qt) = qtext {
            let or = format!("title.ilike.%{}%,description.ilike.%{}%", qt, qt);
            req = req.query(&[("or", or)]);
        }
        if let Some(tags_csv) = q.tags.as_deref() {
            let items: Vec<String> = tags_csv.split(|c| c == ',' || c == '、' || c == ';' || c == '|').map(|s| s.trim().to_string()).filter(|s| !s.is_empty()).collect();
            if !items.is_empty() {
                let joined = items.join(",");
                req = req.query(&[("tags", format!("cs.{{{}}}", joined))]);
            }
        }
        if let Some(tags_any_csv) = q.tags_any.as_deref() {
            let items: Vec<String> = tags_any_csv.split(|c| c == ',' || c == '、' || c == ';' || c == '|').map(|s| s.trim().to_string()).filter(|s| !s.is_empty()).collect();
            if !items.is_empty() {
                let joined = items.join(",");
                req = req.query(&[("tags", format!("ov.{{{}}}", joined))]);
            }
        }
        if let Ok(resp) = req.send().await {
            let headers_clone = resp.headers().clone();
            if let Ok(json) = resp.json::<Vec<HistoryEvent>>().await {
                let mut response = Json(json).into_response();
                if let Some(range) = headers_clone.get("content-range") {
                    response.headers_mut().insert("Content-Range", range.clone());
                    let total = range.to_str().ok().and_then(|s| s.split('/').nth(1)).unwrap_or("");
                    if let Ok(hv) = HeaderValue::from_str(total) {
                        response.headers_mut().insert("X-Total-Count", hv);
                    }
                }
                let expose = HeaderValue::from_static("Content-Range,X-Total-Count");
                response.headers_mut().insert("Access-Control-Expose-Headers", expose);
                return response;
            }
        }
    }

    let store = HISTORY_STORE.read().unwrap();
    let mut filtered: Vec<HistoryEvent> = store.iter().cloned()
        .filter(|e| e.year >= start && e.year <= end)
        .filter(|e| match category { Some(c) => e.category.as_deref() == Some(c), None => true })
        .collect();

    // Append curated upcoming major events (offline file)
    if let Ok(file) = File::open("backend/data/major_events.json") {
        let reader = BufReader::new(file);
        #[derive(Deserialize)]
        struct Wrapper { events: Vec<HistoryEvent> }
        if let Ok(wrapper) = serde_json::from_reader::<_, Wrapper>(reader) {
            let mut extra: Vec<HistoryEvent> = wrapper.events.into_iter()
                .filter(|e| e.year >= start && e.year <= end)
                .filter(|e| match category { Some(c) => e.category.as_deref() == Some(c), None => true })
                .collect();
            filtered.append(&mut extra);
        }
    }
    if let Some(qt) = qtext {
        let qt_lower = qt.to_lowercase();
        filtered = filtered.into_iter().filter(|e| {
            e.title.to_lowercase().contains(&qt_lower) || e.description.to_lowercase().contains(&qt_lower)
        }).collect();
    }
    if let Some(tags_csv) = q.tags.as_deref() {
        let required: Vec<String> = tags_csv.split(|c| c == ',' || c == '、' || c == ';' || c == '|').map(|s| s.trim().to_string()).filter(|s| !s.is_empty()).collect();
        if !required.is_empty() {
            filtered = filtered.into_iter().filter(|e| {
                if let Some(ts) = &e.tags { required.iter().all(|t| ts.iter().any(|x| x == t)) } else { false }
            }).collect();
        }
    }
    let total = filtered.len();
    let offset = (page.saturating_sub(1)) * page_size;
    let end_idx = offset.saturating_add(page_size).min(total);
    let page_slice = if offset < total { filtered[offset..end_idx].to_vec() } else { Vec::new() };
    let mut response = Json(page_slice).into_response();
    if total > 0 {
        let start_idx = if end_idx == 0 { 0 } else { offset };
        let content_range_val = format!("{}-{}{}{}", start_idx, end_idx.saturating_sub(1), "/", total);
        if let Ok(cr) = HeaderValue::from_str(&content_range_val) {
            response.headers_mut().insert("Content-Range", cr);
        }
        if let Ok(tc) = HeaderValue::from_str(&total.to_string()) {
            response.headers_mut().insert("X-Total-Count", tc);
        }
    } else {
        let content_range_val = format!("{}", format!("*/{}", total));
        if let Ok(cr) = HeaderValue::from_str(&content_range_val) {
            response.headers_mut().insert("Content-Range", cr);
        }
        if let Ok(tc) = HeaderValue::from_str(&total.to_string()) {
            response.headers_mut().insert("X-Total-Count", tc);
        }
    }
    let expose = HeaderValue::from_static("Content-Range,X-Total-Count");
    response.headers_mut().insert("Access-Control-Expose-Headers", expose);
    response
}

#[derive(Deserialize)]
struct ImportQuery { path: String }
#[derive(Serialize)]
struct ImportResp { success: bool, count: usize }

#[axum::debug_handler]
async fn import_history_excel(Query(q): Query<ImportQuery>) -> Json<ImportResp> {
    let result = task::spawn_blocking(move || {
        use calamine::{Reader, Xlsx, DataType, open_workbook};
        let mut events: Vec<HistoryEvent> = Vec::new();
        let wb = open_workbook::<Xlsx<_>, _>(&q.path);
        if let Ok(mut workbook) = wb {
            for sheet_name in workbook.sheet_names().iter() {
                if let Ok(range) = workbook.worksheet_range(sheet_name) {
                    // find header row by keywords within first 20 rows
                    let mut header_row_idx: Option<usize> = None;
                    let mut headers: Vec<String> = Vec::new();
                    for (r, row) in range.rows().enumerate() {
                        if r > 20 { break; }
                        let joined = row.iter().map(|c| c.to_string()).collect::<Vec<_>>().join("|");
                        if joined.contains("年份") || joined.to_lowercase().contains("year") || joined.contains("事件") || joined.contains("标题") {
                            header_row_idx = Some(r);
                            headers = row.iter().map(|c| c.to_string()).collect();
                            break;
                        }
                    }
                    let start_r = header_row_idx.unwrap_or(0);
                    for (r, row) in range.rows().enumerate() {
                        if r <= start_r { continue; }
                        let mut year: Option<i32> = None;
                        let mut title: Option<String> = None;
                        let mut description: Option<String> = None;
                        let mut category: Option<String> = None;
                        let mut tags: Option<Vec<String>> = None;
                        let mut dynasty: Option<String> = None;
                        let mut person: Option<String> = None;
                        for (i, cell) in row.iter().enumerate() {
                            let h: String = headers.get(i).map_or(String::new(), |s: &String| s.trim().to_lowercase());
                            let v = match cell { DataType::Empty => String::new(), DataType::Float(f) => (*f as i32).to_string(), _ => cell.to_string() };
                            match h.as_str() {
                                "year" | "年份" | "纪年" | "公元年" | "西元年" => { year = v.trim().parse::<i32>().ok(); }
                                "title" | "事件" | "标题" | "事件名" | "事件标题" => { let s = v.trim().to_string(); if !s.is_empty() { title = Some(s); } }
                                "description" | "描述" | "备注" | "说明" => { let s = v.trim().to_string(); if !s.is_empty() { description = Some(s); } }
                                "category" | "分类" | "类别" => { let s = v.trim().to_string(); if !s.is_empty() { category = Some(s); } }
                                "tags" | "标签" | "关键字" => {
                                    let vec = v.split(|c| c == ',' || c == '、' || c == ';' || c == '|' )
                                        .map(|s| s.trim().to_string())
                                        .filter(|s| !s.is_empty())
                                        .collect::<Vec<_>>();
                                    if !vec.is_empty() { tags = Some(vec); }
                                }
                                "dynasty" | "朝代" => { let s = v.trim().to_string(); if !s.is_empty() { dynasty = Some(s); } }
                                "person" | "人物" | "皇帝" | "君主" => { let s = v.trim().to_string(); if !s.is_empty() { person = Some(s); } }
                                _ => {}
                            }
                        }
                        if description.as_deref().unwrap_or("").is_empty() {
                            let mut desc = String::new();
                            if let Some(d) = dynasty.clone() { if !d.is_empty() { desc.push_str(&d); } }
                            if let Some(p) = person.clone() { if !p.is_empty() { if !desc.is_empty() { desc.push(' '); } desc.push_str(&p); } }
                            if !desc.is_empty() { description = Some(desc); }
                        }
                        if let (Some(y), Some(t)) = (year, title.clone()) {
                            events.push(HistoryEvent { year: y, title: t, description: description.unwrap_or_default(), category: category.clone(), tags: tags.clone() });
                        }
                    }
                }
            }
            let out_path = "backend/data/history.json";
            if let Ok(mut f) = File::create(out_path) {
                let wrapper = serde_json::json!({ "events": events });
                let _ = f.write_all(wrapper.to_string().as_bytes());
            }
            Ok(events)
        } else { Err(()) }
    }).await.unwrap_or(Err(()));

    if let Ok(events) = result {
        let count = events.len();
        {
            let mut store = HISTORY_STORE.write().unwrap();
            *store = events.clone();
        }

        let supabase_url = env::var("SUPABASE_URL").ok();
        let supabase_key = env::var("SUPABASE_KEY").ok();
        if let (Some(url), Some(key)) = (supabase_url, supabase_key) {
            let client = Client::new();
            let _ = client
                .post(format!("{}/rest/v1/history_events?on_conflict=year,title", url))
                .header("apikey", &key)
                .header("Authorization", format!("Bearer {}", key))
                .header("Content-Type", "application/json")
                .header("Prefer", "resolution=merge-duplicates")
                .body(serde_json::to_string(&events).unwrap_or("[]".to_string()))
                .send().await;
        }
        Json(ImportResp { success: true, count })
    } else {
        Json(ImportResp { success: false, count: 0 })
    }
}
#[derive(Deserialize)]
struct ImportMappingQuery { path: String }
#[derive(Serialize)]
struct ImportMappingResp { success: bool, count: usize }

#[axum::debug_handler]
async fn import_mapping_excel(Query(q): Query<ImportMappingQuery>) -> Json<ImportMappingResp> {
    let result = task::spawn_blocking(move || {
        use calamine::{Reader, Xlsx, DataType, open_workbook};
        let mut rows: Vec<huangji_core::data::YearRecord> = Vec::new();
        let wb = open_workbook::<Xlsx<_>, _>(&q.path);
        if let Ok(mut workbook) = wb {
            for sheet_name in workbook.sheet_names().iter() {
                if let Ok(range) = workbook.worksheet_range(sheet_name) {
                    let mut header_row_idx: Option<usize> = None;
                    let mut headers: Vec<String> = Vec::new();
                    for (r, row) in range.rows().enumerate() {
                        if r > 20 { break; }
                        let joined = row.iter().map(|c| c.to_string()).collect::<Vec<_>>().join("|");
                        if joined.contains("年份") || joined.to_lowercase().contains("year") || joined.contains("元") || joined.contains("会") || joined.contains("运") || joined.contains("世") || joined.contains("旬") {
                            header_row_idx = Some(r);
                            headers = row.iter().map(|c| c.to_string()).collect();
                            break;
                        }
                    }
                    let start_r = header_row_idx.unwrap_or(0);
                    for (r, row) in range.rows().enumerate() {
                        if r <= start_r { continue; }
                        let mut rec = huangji_core::data::YearRecord {
                            gregorian_year: 0,
                            ganzhi: String::new(),
                            nian_hexagram: String::new(),
                            dynasty: String::new(),
                            person: String::new(),
                            yuan_raw: String::new(),
                            hui_raw: String::new(),
                            yun_raw: String::new(),
                            shi_raw: String::new(),
                            xun_raw: String::new(),
                        };
                        for (i, cell) in row.iter().enumerate() {
                            let h: String = headers.get(i).map_or(String::new(), |s: &String| s.trim().to_lowercase());
                            let v = match cell { DataType::Empty => String::new(), DataType::Float(f) => (*f as i32).to_string(), _ => cell.to_string() };
                            match h.as_str() {
                                "year" | "年份" | "公元年" | "西元年" => { rec.gregorian_year = v.trim().parse::<i32>().unwrap_or(0); }
                                "ganzhi" | "干支" | "年干支" | "干支年" => { rec.ganzhi = v.trim().to_string(); }
                                "hexagram" | "年卦" | "卦" => { rec.nian_hexagram = v.trim().to_string(); }
                                "dynasty" | "朝代" => { rec.dynasty = v.trim().to_string(); }
                                "person" | "人物" | "皇帝" | "君主" => { rec.person = v.trim().to_string(); }
                                "yuan" | "元" | "元期" => { rec.yuan_raw = v.trim().to_string(); }
                                "hui" | "会" | "会期" => { rec.hui_raw = v.trim().to_string(); }
                                "yun" | "运" | "大运" | "主运" => { rec.yun_raw = v.trim().to_string(); }
                                "shi" | "世" => { rec.shi_raw = v.trim().to_string(); }
                                "xun" | "旬" => { rec.xun_raw = v.trim().to_string(); }
                                _ => {}
                            }
                        }
                        if rec.gregorian_year != 0 { rows.push(rec); }
                    }
                }
            }
            let out_path = "huangji_core/data/year_mapping.json";
            if let Ok(mut f) = File::create(out_path) { let _ = f.write_all(serde_json::to_string(&rows).unwrap_or("[]".to_string()).as_bytes()); }
            Ok((rows, out_path.to_string()))
        } else { Err(()) }
    }).await.unwrap_or(Err(()));

    if let Ok((rows, out_path)) = result {
        let _ = data::load_data(&out_path);
        let count = rows.len();
        Json(ImportMappingResp { success: true, count })
    } else {
        Json(ImportMappingResp { success: false, count: 0 })
    }
}

#[axum::debug_handler]
async fn import_mapping_json(Query(q): Query<ImportMappingQuery>) -> Json<ImportMappingResp> {
    let result = task::spawn_blocking(move || {
        #[allow(dead_code)]
        #[derive(Deserialize)]
        struct IncomingRecord {
            year: i32,
            ganzhi: Option<String>,
            annual_hex: Option<String>,
            dynasty: Option<String>,
            figure: Option<String>,
            hui_label: Option<String>,
            yun_label: Option<String>,
            yun_range_hex: Option<String>,
            yun_base_hex: Option<String>,
            xun_hex: Option<String>,
            event: Option<String>,
        }
        let file = File::open(&q.path).map_err(|_| ())?;
        let reader = BufReader::new(file);
        let incoming: Vec<IncomingRecord> = serde_json::from_reader(reader).map_err(|_| ())?;
        let rows: Vec<huangji_core::data::YearRecord> = incoming.into_iter().map(|r| {
            huangji_core::data::YearRecord {
                gregorian_year: r.year,
                ganzhi: r.ganzhi.unwrap_or_default(),
                nian_hexagram: r.annual_hex.unwrap_or_default(),
                dynasty: r.dynasty.unwrap_or_default(),
                person: r.figure.unwrap_or_default(),
                yuan_raw: r.yun_base_hex.unwrap_or_default(),
                hui_raw: r.hui_label.unwrap_or_default(),
                yun_raw: r.yun_label.unwrap_or_else(|| r.yun_range_hex.unwrap_or_default()),
                shi_raw: String::new(),
                xun_raw: r.xun_hex.unwrap_or_default(),
            }
        }).collect();
        let out_path = "huangji_core/data/year_mapping.json";
        if let Ok(mut f) = File::create(out_path) {
            let _ = f.write_all(serde_json::to_string(&rows).unwrap_or("[]".to_string()).as_bytes());
        }
        Ok((rows, out_path.to_string()))
    }).await.unwrap_or(Err(()));

    if let Ok((rows, out_path)) = result {
        let _ = data::load_data(&out_path);
        let count = rows.len();
        Json(ImportMappingResp { success: true, count })
    } else {
        Json(ImportMappingResp { success: false, count: 0 })
    }
}
#[derive(Deserialize)]
struct InspectQuery { path: String }
#[derive(Serialize)]
struct InspectResp { sheet_names: Vec<String>, samples: Vec<Vec<String>> }

#[axum::debug_handler]
async fn inspect_excel(Query(q): Query<InspectQuery>) -> Json<InspectResp> {
    use calamine::{Reader, Xlsx, open_workbook, DataType};
    let mut names: Vec<String> = Vec::new();
    let mut samples: Vec<Vec<String>> = Vec::new();
    if let Ok(mut workbook) = open_workbook::<Xlsx<_>, _>(&q.path) {
        names = workbook.sheet_names().clone();
        for sheet in workbook.sheet_names().iter().take(1) {
            if let Ok(range) = workbook.worksheet_range(sheet) {
                for (_i, row) in range.rows().enumerate().take(5) {
                    let mut vals: Vec<String> = Vec::new();
                    for cell in row.iter().take(8) {
                        let v = match cell { DataType::Empty => String::new(), DataType::Float(f) => (*f as i32).to_string(), _ => cell.to_string() };
                        vals.push(v);
                    }
                    samples.push(vals);
                }
            }
        }
    }
    Json(InspectResp { sheet_names: names, samples })
}

#[derive(Deserialize)]
struct GetMappingQuery { year: i32 }
#[derive(Serialize)]
struct GetMappingResp { record: Option<huangji_core::data::YearRecord> }

#[axum::debug_handler]
async fn get_mapping_by_year(Query(q): Query<GetMappingQuery>) -> Json<GetMappingResp> {
    let rec = huangji_core::data::get_year_record(q.year);
    Json(GetMappingResp { record: rec })
}

#[derive(Deserialize)]
struct RelatedQuery { year: i32, mode: Option<String>, limit: Option<usize>, category: Option<String>, tags: Option<String>, tags_any: Option<String> }
#[derive(Serialize, Clone)]
struct RelatedItem { year: i32, title: String, dynasty: Option<String>, person: Option<String> }

#[axum::debug_handler]
async fn get_related_history(Query(q): Query<RelatedQuery>) -> Json<Vec<RelatedItem>> {
    let year = q.year;
    let mode = q.mode.as_deref().unwrap_or("yun");
    let limit = q.limit.unwrap_or(3);
    let category = q.category.as_deref();
    let tags_all: Option<Vec<String>> = q.tags.as_deref().map(|s| s.split(|c| c == ',' || c == '、' || c == ';' || c == '|').map(|x| x.trim().to_string()).filter(|x| !x.is_empty()).collect());
    let tags_any: Option<Vec<String>> = q.tags_any.as_deref().map(|s| s.split(|c| c == ',' || c == '、' || c == ';' || c == '|').map(|x| x.trim().to_string()).filter(|x| !x.is_empty()).collect());

    // Helper: attach dynasty/person from mapping
    let enrich = |y: i32, title: String| {
        let rec = huangji_core::data::get_year_record(y);
        RelatedItem { year: y, title, dynasty: rec.as_ref().map(|r| r.dynasty.clone()), person: rec.as_ref().map(|r| r.person.clone()) }
    };

    let mut items: Vec<RelatedItem> = Vec::new();
    if mode == "annual_hex" {
        if let Some(rec) = huangji_core::data::get_year_record(year) {
            let target_hex = rec.nian_hexagram;
            let store = huangji_core::data::YEAR_DATA.read().unwrap();
            let mut years: Vec<i32> = store.iter()
                .filter(|(_y, r)| r.nian_hexagram == target_hex)
                .map(|(y, _)| *y)
                .collect();
            years.sort_by_key(|y| (y - year).abs());
            let hist = HISTORY_STORE.read().unwrap();
            for y in years.into_iter().take(limit) {
                if let Some(ev) = hist.iter().find(|e| e.year == y) {
                    if let Some(c) = category { if ev.category.as_deref() != Some(c) { continue; } }
                    if let Some(req) = &tags_all {
                        if let Some(ts) = &ev.tags { if !req.iter().all(|t| ts.iter().any(|x| x == t)) { continue; } } else { continue; }
                    }
                    if let Some(any) = &tags_any {
                        if let Some(ts) = &ev.tags { if !any.iter().any(|t| ts.iter().any(|x| x == t)) { continue; } } else { continue; }
                    }
                    items.push(enrich(y, ev.title.clone()));
                } else {
                    if category.is_some() || tags_all.as_ref().map(|v| !v.is_empty()).unwrap_or(false) || tags_any.as_ref().map(|v| !v.is_empty()).unwrap_or(false) {
                        continue;
                    }
                    items.push(enrich(y, target_hex.clone()));
                }
            }
        }
    } else {
        // default: within same yun range
        let tl = algorithm::get_timeline_info(year);
        let start = tl.current.yun.start_year;
        let end = tl.current.yun.end_year;
        let hist = HISTORY_STORE.read().unwrap();
        let mut candidates: Vec<HistoryEvent> = hist.iter().cloned()
            .filter(|e| e.year >= start && e.year <= end)
            .filter(|e| match category { Some(c) => e.category.as_deref() == Some(c), None => true })
            .collect();
        if let Some(req) = &tags_all {
            candidates = candidates.into_iter().filter(|e| {
                if let Some(ts) = &e.tags { req.iter().all(|t| ts.iter().any(|x| x == t)) } else { false }
            }).collect();
        }
        if let Some(any) = &tags_any {
            candidates = candidates.into_iter().filter(|e| {
                if let Some(ts) = &e.tags { any.iter().any(|t| ts.iter().any(|x| x == t)) } else { false }
            }).collect();
        }
        candidates.sort_by_key(|e| (e.year - year).abs());
        for e in candidates.into_iter().take(limit) {
            items.push(enrich(e.year, e.title));
        }
    }
    Json(items)
}
fn init_celestial_hashes() {
    let path = "backend/data/celestial/hashes.json";
    let mut map: HashMap<String, String> = HashMap::new();
    if let Ok(bytes) = std::fs::read(path) {
        if let Ok(json) = serde_json::from_slice::<HashMap<String, String>>(&bytes) {
            map = json;
        }
    }
    if map.is_empty() {
        map.insert("cultures/cn.json".to_string(), CELESTIAL_HASH_CN.to_string());
        map.insert("stars.6.json".to_string(), CELESTIAL_HASH_STARS_6.to_string());
        map.insert("constellations.json".to_string(), CELESTIAL_HASH_CONSTELLATIONS.to_string());
        map.insert("constellations.lines.json".to_string(), CELESTIAL_HASH_CONSTELLATIONS_LINES.to_string());
        map.insert("constellations.bounds.json".to_string(), CELESTIAL_HASH_CONSTELLATIONS_BOUNDS.to_string());
        map.insert("planets.json".to_string(), CELESTIAL_HASH_PLANETS.to_string());
        map.insert("mw.json".to_string(), CELESTIAL_HASH_MW.to_string());
        if let Some(parent) = std::path::Path::new(path).parent() { let _ = std::fs::create_dir_all(parent); }
        let _ = std::fs::write(path, serde_json::to_vec(&map).unwrap_or_default());
    }
    {
        let mut store = CELESTIAL_HASHES.write().unwrap();
        *store = map;
    }
}
#[axum::debug_handler]
async fn preload_cache() -> Json<bool> {
    use axum::http::StatusCode;
    let allowed = [
        "stars.6.json",
        "constellations.json",
        "constellations.lines.json",
        "constellations.bounds.json",
        "planets.json",
        "mw.json",
        // cultures kept optional; will try cn and iau if available
    ];
    let mut ok = true;
    let client = Client::new();
    let mut roots = get_env_roots();
    if roots.is_empty() {
        roots = vec![
            "https://raw.githubusercontent.com/ofrohn/celestial/master/data/".to_string(),
            "https://cdn.jsdelivr.net/gh/ofrohn/celestial@master/data/".to_string(),
            "https://fastly.jsdelivr.net/gh/ofrohn/celestial@master/data/".to_string(),
            "https://ofrohn.github.io/data/".to_string(),
        ];
    }
    for clean in allowed.iter() {
        let mut fetched = false;
        for r in roots.iter() {
            let url = format!("{}{}", r, clean);
            if let Ok(resp) = client.get(url).send().await {
                let headers_clone = resp.headers().clone();
                if resp.status() == StatusCode::OK {
                    if let Ok(bytes) = resp.bytes().await {
                        let buf = bytes.to_vec();
                        if !verify_hash(clean, &buf) { continue; }
                        let cache_root = "backend/data/celestial";
                        let cache_path = format!("{}/{}", cache_root, clean);
                        if let Some(parent) = std::path::Path::new(&cache_path).parent() { let _ = std::fs::create_dir_all(parent); }
                        let _ = std::fs::write(&cache_path, &buf);
                        let mut hasher = Sha256::new(); hasher.update(&buf); let got = format!("{:x}", hasher.finalize());
                        let etag_hdr = headers_clone.get("etag").and_then(|h| h.to_str().ok()).map(|s| s.to_string());
                        let last_mod_hdr = headers_clone.get("last-modified").and_then(|h| h.to_str().ok()).map(|s| s.to_string());
                        update_cache_index(clean, &got, r, buf.len(), etag_hdr, last_mod_hdr);
                        fetched = true; break;
                    }
                }
            }
        }
        ok &= fetched;
    }
    // optional cultures
    for culture in ["cultures/cn.json", "cultures/iau.json"].iter() {
        let mut fetched = false;
        for r in roots.iter() {
            let url = format!("{}{}", r, culture);
            if let Ok(resp) = client.get(url).send().await {
                let headers_clone = resp.headers().clone();
                if resp.status() == StatusCode::OK {
                    if let Ok(bytes) = resp.bytes().await {
                        let buf = bytes.to_vec();
                        let cache_root = "backend/data/celestial";
                        let cache_path = format!("{}/{}", cache_root, culture);
                        if let Some(parent) = std::path::Path::new(&cache_path).parent() { let _ = std::fs::create_dir_all(parent); }
                        let _ = std::fs::write(&cache_path, &buf);
                        let mut hasher = Sha256::new(); hasher.update(&buf); let got = format!("{:x}", hasher.finalize());
                        let etag_hdr = headers_clone.get("etag").and_then(|h| h.to_str().ok()).map(|s| s.to_string());
                        let last_mod_hdr = headers_clone.get("last-modified").and_then(|h| h.to_str().ok()).map(|s| s.to_string());
                        update_cache_index(culture, &got, r, buf.len(), etag_hdr, last_mod_hdr);
                        fetched = true; break;
                    }
                }
            }
        }
        ok &= fetched;
    }
    Json(ok)
}
