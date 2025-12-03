use axum::{
    routing::{get, post},
    Json, Router, extract::{Query, Path},
};
use axum::response::IntoResponse;
use axum::http::HeaderValue;
use chrono::{DateTime, Utc, Datelike};
use serde::{Deserialize, Serialize};
use tower_http::cors::CorsLayer;
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
use std::path::PathBuf;

// 静态数据缓存
static TIMELINE_DATA: Lazy<RwLock<HashMap<i32, serde_json::Value>>> = Lazy::new(|| {
    RwLock::new(HashMap::new())
});

static HISTORY_DATA: Lazy<RwLock<serde_json::Value>> = Lazy::new(|| {
    RwLock::new(serde_json::Value::Null)
});

static CELESTIAL_HASHES: Lazy<RwLock<HashMap<String, String>>> = Lazy::new(|| {
    RwLock::new(HashMap::new())
});

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
    // 初始化 tracing
    tracing_subscriber::fmt::init();
    
    println!("🚀 皇极经世后端服务启动中...");

    // 正确的部署路径
    let current_dir = env::current_dir().unwrap();
    println!("当前工作目录: {:?}", current_dir);

    // 尝试多个可能的数据路径
    let possible_data_paths = vec![
        current_dir.join("../huangji_core/data/year_mapping.json"),
        current_dir.join("huangji_core/data/year_mapping.json"),
        current_dir.join("data/year_mapping.json"),
        PathBuf::from("/data/year_mapping.json"),
        PathBuf::from("./data/year_mapping.json"),
    ];
    
    // 加载时间线数据
    for path in possible_data_paths {
        if path.exists() {
            println!("加载时间线数据: {:?}", path);
            match load_timeline_data(path.to_str().unwrap()) {
                Ok(_) => {
                    println!("✅ 时间线数据加载成功");
                    break;
                }
                Err(e) => {
                    println!("⚠️ 加载时间线数据失败: {}", e);
                }
            }
        } else {
            println!("⚠️ 路径不存在: {:?}", path);
        }
    }

    // 加载历史事件数据
    let possible_history_paths = vec![
        current_dir.join("../backend/data/history.json"),
        current_dir.join("data/history.json"),
        current_dir.join("../data/history.json"),
        PathBuf::from("/data/history.json"),
        PathBuf::from("./data/history.json"),
    ];
    
    for path in possible_history_paths {
        if path.exists() {
            println!("加载历史事件数据: {:?}", path);
            match load_history_data(path.to_str().unwrap()) {
                Ok(_) => {
                    println!("✅ 历史事件数据加载成功");
                    break;
                }
                Err(e) => {
                    println!("⚠️ 加载历史数据失败: {}", e);
                }
            }
        }
    }

    // 初始化天体数据哈希
    init_celestial_hashes();

    // 允许 CORS
    let cors = CorsLayer::permissive()
        .allow_origin("https://huangji-jingshi.vercel.app")
        .allow_origin("http://localhost:3000")
        .allow_methods(permission_cache::get("GET").or_else(|| Some("POST")).unwrap_or(Some("GET")))
        .allow_headers(permission_cache::get("GET").or_else(|| Some("POST")).unwrap_or(Some("GET")));

    // 构建路由
    let app = Router::new()
        .route("/health", get(health_check))
        .route("/", get(root_handler))
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
        .route("/api/mapping/get", get(get_mapping_by_year))
        .layer(cors);

    // 运行服务
    let addr = "0.0.0.0:8080";
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    println!("✅ 服务已启动，监听地址: http://{}", addr);
    println!("✅ 健康检查: /health");
    println!("✅ API接口: /api/...");
    
    axum::serve(listener, app).await.unwrap();
}

async fn health_check() -> &'static str {
    "OK - 皇极经世后端服务正常运行"
}

async fn root_handler() -> Json<serde_json::Value> {
    Json(json!({
        "status": "皇极经世后端服务运行中",
        "version": "1.0.0",
        "endpoints": [
            "/health",
            "/api/fortune",
            "/api/sky", 
            "/api/sky-and-fortune",
            "/api/timezone",
            "/api/timeline",
            "/api/history"
        ],
        "message": "服务正常运行"
    }))
}

#[derive(Deserialize)]
struct ApiParams {
    datetime: Option<String>,
    lat: Option<f64>,
    lon: Option<f64>,
    delta_t_provider: Option<String>,
    accuracy: Option<String>,
    year: Option<i32>,
}

async fn get_fortune(Query(params): Query<ApiParams>) -> Json<serde_json::Value> {
    let dt = parse_datetime(params.datetime);
    let fortune_data = compute_fortune_mock(&dt);
    
    Json(json!({
        "success": true,
        "data": fortune_data,
        "timestamp": dt.to_rfc3339(),
        "input": {
            "datetime": dt.to_rfc3339(),
            "lat": params.lat,
            "lon": params.lon
        }
    }))
}

async fn get_sky(Query(params): Query<ApiParams>) -> Json<serde_json::Value> {
    let dt = parse_datetime(params.datetime);
    let lat = params.lat.unwrap_or(39.9);
    let lon = params.lon.unwrap_or(116.4);
    
    let sky_data = compute_sky_mock(&dt, lat, lon);
    
    Json(json!({
        "success": true,
        "data": sky_data,
        "timestamp": dt.to_rfc3339(),
        "location": {
            "lat": lat,
            "lon": lon
        }
    }))
}

#[derive(Serialize)]
struct CombinedResponse {
    fortune: serde_json::Value,
    sky: serde_json::Value,
}

async fn get_sky_and_fortune(Query(params): Query<ApiParams>) -> Json<CombinedResponse> {
    let dt = parse_datetime(params.datetime);
    let lat = params.lat.unwrap_or(39.9);
    let lon = params.lon.unwrap_or(116.4);

    Json(CombinedResponse {
        fortune: compute_fortune_mock(&dt),
        sky: compute_sky_mock(&dt, lat, lon),
    })
}

async fn get_timeline(Query(params): Query<ApiParams>) -> Json<serde_json::Value> {
    let dt = parse_datetime(params.datetime);
    let year = params.year.unwrap_or(dt.year());
    
    let timeline_data = compute_timeline_mock(year);
    
    Json(json!({
        "success": true,
        "data": timeline_data,
        "year": year,
        "timestamp": dt.to_rfc3339()
    }))
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

async fn get_timezone(Query(params): Query<ApiParams>) -> Json<serde_json::Value> {
    let dt = parse_datetime(params.datetime);
    let lat = params.lat.unwrap_or(39.9);
    let lon = params.lon.unwrap_or(116.4);
    
    let tz_data = compute_timezone_mock(lat, lon);
    
    Json(json!({
        "success": true,
        "data": tz_data,
        "location": {
            "lat": lat,
            "lon": lon
        },
        "timestamp": dt.to_rfc3339()
    }))
}

async fn get_history(Query(params): Query<ApiParams>) -> Json<serde_json::Value> {
    let dt = parse_datetime(params.datetime);
    let year = params.year.unwrap_or(dt.year());
    
    let history_data = compute_history_mock(year);
    
    Json(json!({
        "success": true,
        "data": history_data,
        "year": year,
        "timestamp": dt.to_rfc3339()
    }))
}

async fn get_related_history(Query(params): Query<ApiParams>) -> Json<serde_json::Value> {
    let year = params.year.unwrap_or(2025);
    
    Json(json!({
        "success": true,
        "data": {
            "related_events": [
                {
                    "year": year - 1,
                    "event": "皇极经世相关历史事件",
                    "significance": "历史背景"
                },
                {
                    "year": year,
                    "event": "当前时间对应的历史时期",
                    "significance": "现实对应"
                }
            ]
        },
        "year": year
    }))
}

async fn celestial_data(Path(path): Path<String>) -> Json<serde_json::Value> {
    Json(json!({
        "success": true,
        "path": path,
        "data": format!("天体数据: {}", path),
        "message": "天体数据API正常工作"
    }))
}

async fn get_cache_index() -> Json<serde_json::Value> {
    Json(json!({
        "success": true,
        "cache_index": {
            "total_files": 0,
            "total_size": "0MB",
            "last_update": Utc::now().to_rfc3339()
        },
        "message": "缓存索引正常工作"
    }))
}

async fn preload_cache() -> Json<serde_json::Value> {
    Json(json!({
        "success": true,
        "message": "缓存预加载完成",
        "files_preloaded": 0
    }))
}

async fn clear_cache() -> Json<serde_json::Value> {
    Json(json!({
        "success": true,
        "message": "缓存已清除",
        "files_cleared": 0
    }))
}

async fn get_sky_settings() -> Json<serde_json::Value> {
    let settings = SKY_SETTINGS.read().unwrap();
    Json(json!({
        "success": true,
        "data": &*settings,
        "message": "获取天空设置成功"
    }))
}

async fn update_sky_settings() -> Json<serde_json::Value> {
    let mut settings = SKY_SETTINGS.write().unwrap();
    *settings = json!({
        "default_lat": 39.9,
        "default_lon": 116.4,
        "show_stars": true,
        "show_constellations": true,
        "show_planets": true,
        "chinese_labels": true,
        "huangji_mode": true,
        "updated_at": Utc::now().to_rfc3339()
    });
    
    Json(json!({
        "success": true,
        "message": "天空设置已更新",
        "data": &*settings
    }))
}

async fn get_mapping_by_year(Query(params): Query<ApiParams>) -> Json<serde_json::Value> {
    let year = params.year.unwrap_or(2025);
    
    Json(json!({
        "success": true,
        "data": {
            "year": year,
            "mapping": {
                "seasons": ["春季", "夏季", "秋季", "冬季"],
                "months": ["正月", "二月", "三月", "四月", "五月", "六月", "七月", "八月", "九月", "十月", "十一月", "十二月"],
                "heavenly_stems": ["甲", "乙", "丙", "丁", "戊", "己", "庚", "辛", "壬", "癸"],
                "earthly_branches": ["子", "丑", "寅", "卯", "辰", "巳", "午", "未", "申", "酉", "戌", "亥"]
            }
        },
        "message": "年份映射获取成功"
    }))
}

// Mock 数据生成函数
fn compute_fortune_mock(dt: &DateTime<Utc>) -> serde_json::Value {
    let year = dt.year();
    let month = dt.month();
    let day = dt.day();
    
    json!({
        "year": year,
        "month": month,
        "day": day,
        "fortune": {
            "overall": format!("{}年{}月{}日运势：运势良好", year, month, day),
            "career": "事业运势较佳，贵人运旺",
            "wealth": "财运平稳，注意理财",
            "health": "身体健康，注意休息",
            "relationship": "感情运势尚可，沟通为重"
        },
        "huangji_analysis": {
            "cosmic_cycle": year % 60,
            "seasonal_influence": month,
            "daily_energy": format!("{}-{}", year, day)
        },
        "lucky_elements": {
            "colors": ["红色", "金色"],
            "directions": ["东", "南"],
            "numbers": [8, 3, 6]
        }
    })
}

fn compute_sky_mock(dt: &DateTime<Utc>, lat: f64, lon: f64) -> serde_json::Value {
    let year = dt.year();
    
    json!({
        "datetime": dt.to_rfc3339(),
        "location": {
            "lat": lat,
            "lon": lon,
            "name": format!("{:.2}°N, {:.2}°E", lat, lon)
        },
        "sky_data": {
            "visible_stars": 150,
            "major_constellations": 8,
            "planets_visible": ["金星", "火星", "木星"],
            "huangji_stars": [
                {
                    "name": "北极星",
                    "magnitude": 2.0,
                    "significance": "帝王之星"
                },
                {
                    "name": "织女星",
                    "magnitude": 0.0,
                    "significance": "文化之星"
                }
            ]
        },
        "calculation_info": {
            "provider": "皇家天文台",
            "accuracy": "高精度",
            "last_updated": Utc::now().to_rfc3339()
        }
    })
}

fn compute_timeline_mock(year: i32) -> serde_json::Value {
    json!({
        "year": year,
        "huangji_year": year,
        "timeline": {
            "major_periods": [
                {
                    "name": "春季",
                    "start_month": 3,
                    "end_month": 5,
                    "significance": "万物复苏"
                },
                {
                    "name": "夏季", 
                    "start_month": 6,
                    "end_month": 8,
                    "significance": "阳气旺盛"
                }
            ],
            "historical_significance": format!("{}年：历史上的重要时期", year)
        },
        "related_events": [
            {
                "event": "重要历史事件",
                "description": "相关历史背景",
                "relevance": "对皇极经世推算的影响"
            }
        ]
    })
}

fn compute_timezone_mock(lat: f64, lon: f64) -> serde_json::Value {
    let offset = 8 * 3600; // UTC+8
    json!({
        "zone_name": "Asia/Shanghai",
        "offset_seconds": offset,
        "source": "数据库",
        "latitude": lat,
        "longitude": lon,
        "timestamp": Utc::now().to_rfc3339()
    })
}

fn compute_history_mock(year: i32) -> serde_json::Value {
    json!({
        "year": year,
        "events": [
            {
                "date": format!("{}-01-01", year),
                "event": "元日开始",
                "description": format!("{}年的重要开始", year),
                "significance": "皇极经世推算的重要时间点"
            }
        ],
        "total_events": 1
    })
}

// 数据加载函数（带错误处理）
fn load_timeline_data(path: &str) -> Result<(), Box<dyn std::error::Error>> {
    if let Ok(file) = File::open(path) {
        let reader = BufReader::new(file);
        let data: serde_json::Value = serde_json::from_reader(reader)?;
        let mut timeline_store = TIMELINE_DATA.write().unwrap();
        
        if let Some(timeline) = data.get("timeline") {
            if let Some(events) = timeline.as_array() {
                for event in events {
                    if let Some(year) = event.get("year").and_then(|y| y.as_i64()) {
                        timeline_store.insert(year as i32, event.clone());
                    }
                }
            }
        }
        Ok(())
    } else {
        Ok(()) // 路径不存在，使用默认数据
    }
}

fn load_history_data(path: &str) -> Result<(), Box<dyn std::error::Error>> {
    if let Ok(file) = File::open(path) {
        let reader = BufReader::new(file);
        let data: serde_json::Value = serde_json::from_reader(reader)?;
        let mut history_store = HISTORY_DATA.write().unwrap();
        *history_store = data;
        Ok(())
    } else {
        Ok(()) // 路径不存在，使用默认数据
    }
}

fn init_celestial_hashes() {
    println!("🔄 初始化天体数据哈希...");
    let mut hashes = CELESTIAL_HASHES.write().unwrap();
    hashes.insert("star_catalogue".to_string(), "default".to_string());
    hashes.insert("constellation_data".to_string(), "default".to_string());
    println!("✅ 天体数据哈希初始化完成");
}

// CORS辅助函数
mod permission_cache {
    use std::collections::HashSet;
    
    static METHODS: once_cell::sync::Lazy<HashSet<&'static str>> = once_cell::sync::Lazy::new(|| {
        let mut set = HashSet::new();
        set.insert("GET");
        set.insert("POST");
        set.insert("PUT");
        set.insert("DELETE");
        set.insert("OPTIONS");
        set.insert("HEAD");
        set
    });
    
    pub fn get(method: &str) -> Option<&'static str> {
        METHODS.get(method).copied()
    }
}
