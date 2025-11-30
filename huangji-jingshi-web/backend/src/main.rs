use axum::{
    routing::get,
    Json, Router, extract::Query,
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
#[derive(Clone)]
struct TzCacheEntry {
    zone_name: Option<String>,
    offset_seconds: i32,
    expires_at: i64,
}
static TIMEZONE_CACHE: Lazy<RwLock<HashMap<String, TzCacheEntry>>> = Lazy::new(|| RwLock::new(HashMap::new()));

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
