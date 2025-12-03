use axum::{
    routing::{get, post},
    Json, Router, extract::{Path, Query},
};
use axum::response::IntoResponse;
use chrono::Utc;
use serde::Deserialize;
use tower_http::cors::CorsLayer;
use std::sync::RwLock;
use std::collections::HashMap;
use std::env;
use serde_json::json;
use std::path::PathBuf;
use once_cell::sync::Lazy;

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

    // 初始化数据加载
    if let Some(path) = &data_path {
        tracing::info!("📂 尝试加载数据文件...");
        let _ = load_data_files(path).await;
    } else {
        tracing::warn!("⚠️ 未找到数据文件，将使用Mock数据");
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
        
        // 静态文件服务
        .route("/static/:file", get(static_handler))
        
        // CORS - 允许所有来源
        .layer(
            CorsLayer::permissive()
        );

    tracing::info!("🌐 启动服务器，端口: {}", port);

    // 启动服务器
    let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{}", port)).await.unwrap();
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
        "/app/data/celestial"
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

// 数据加载函数
async fn load_data_files(data_path: &PathBuf) -> Result<(), Box<dyn std::error::Error>> {
    tracing::info!("📊 开始加载数据文件...");

    // 获取数据根目录
    let data_root = if data_path.to_str().unwrap().contains("celestial") {
        data_path.parent().unwrap_or(data_path)
    } else {
        data_path
    };

    // 加载历史数据
    let history_path = data_root.join("history.json");
    tracing::info!("🔍 尝试加载历史数据: {:?}", history_path);
    if history_path.exists() {
        match load_json_file(&history_path).await {
            Ok(data) => {
                *HISTORY_DATA.write().unwrap() = data;
                tracing::info!("✅ 历史数据加载成功");
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
                if let Some(obj) = data.as_object() {
                    for (key, value) in obj.iter() {
                        if let Ok(year) = key.parse::<i32>() {
                            TIMELINE_DATA.write().unwrap().insert(year, value.clone());
                        }
                    }
                    tracing::info!("✅ 主要事件数据加载成功");
                }
            }
            Err(e) => tracing::warn!("⚠️ 主要事件数据加载失败: {}", e),
        }
    }

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
    Json(json!({
        "status": "ok",
        "message": "皇极经世后端服务正常运行",
        "timestamp": Utc::now().to_rfc3339(),
        "version": "1.0.0-fixed",
        "data_loaded": TIMELINE_DATA.read().unwrap().len() > 0
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

// 天机演算
async fn calculate(Json(payload): Json<serde_json::Value>) -> impl IntoResponse {
    tracing::info!("🔮 收到演算请求: {:?}", payload);

    // 模拟演算过程
    tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;

    let calc_id = format!("calc_{}", Utc::now().timestamp());

    Json(json!({
        "result": "天机演算完成",
        "calculation_id": calc_id,
        "input": payload,
        "output": {
            "ganzhi": "甲子",
            "date": "2025-12-03",
            "fortune": "大吉",
            "stars": json!(["紫微", "天机", "太阳"])
        },
        "timestamp": Utc::now().to_rfc3339(),
        "status": "success"
    }))
}

#[derive(Deserialize)]
struct TimelineQuery {
    datetime: String,
}

#[derive(Deserialize)]
struct SkyFortuneQuery {
    datetime: String,
    lat: Option<f64>,
    lon: Option<f64>,
}

#[derive(Deserialize)]
struct HistoryQuery {
    start: Option<i32>,
    end: Option<i32>,
}

#[derive(Deserialize)]
struct HistoryRelatedQuery {
    year: Option<i32>,
    mode: Option<String>,
    limit: Option<i32>,
}

#[derive(Deserialize)]
struct MappingQuery {
    year: Option<i32>,
}

// 核心 API - 获取天象和运势数据
async fn get_sky_and_fortune(Query(params): Query<SkyFortuneQuery>) -> impl IntoResponse {
    let year: i32 = params.datetime
        .split('-')
        .next()
        .and_then(|s| s.parse().ok())
        .unwrap_or(2025);
    
    let lat = params.lat.unwrap_or(39.9);
    let lon = params.lon.unwrap_or(116.4);
    
    tracing::info!("🌟 获取天象运势: {} @ ({}, {})", params.datetime, lat, lon);
    
    // 返回完整的天象和运势数据，完全匹配前端 CombinedResponse 类型
    Json(json!({
        "sky": {
            "bodies": [
                {"name": "Sun", "ra_deg": 250.5, "dec_deg": -23.2, "alt_deg": 45.0, "az_deg": 180.0, "distance_au": 0.983},
                {"name": "Moon", "ra_deg": 120.3, "dec_deg": 15.6, "alt_deg": 60.0, "az_deg": 120.0, "distance_au": 0.0025},
                {"name": "Mercury", "ra_deg": 245.0, "dec_deg": -20.0, "alt_deg": 42.0, "az_deg": 175.0, "distance_au": 1.2},
                {"name": "Venus", "ra_deg": 280.0, "dec_deg": -25.0, "alt_deg": 30.0, "az_deg": 200.0, "distance_au": 0.7},
                {"name": "Mars", "ra_deg": 100.0, "dec_deg": 20.0, "alt_deg": 55.0, "az_deg": 100.0, "distance_au": 1.5},
                {"name": "Jupiter", "ra_deg": 60.0, "dec_deg": 22.0, "alt_deg": 70.0, "az_deg": 80.0, "distance_au": 5.2},
                {"name": "Saturn", "ra_deg": 340.0, "dec_deg": -10.0, "alt_deg": 25.0, "az_deg": 250.0, "distance_au": 9.5}
            ],
            "note": format!("天象数据 - {} @ ({:.2}, {:.2})", params.datetime, lat, lon),
            "jd": 2460649.0,
            "lst_deg": 45.6,
            "gmst_deg": 123.456,
            "delta_t_sec": 69.184
        },
        "fortune": {
            "yuan": "第1元",
            "hui": "第1会 · 元会",
            "yun": "第6运 · 己运",
            "shi": "第2世 · 丑世",
            "xun": "第2旬 · 甲戌旬",
            "nian_ganzhi": "乙巳年",
            "hexagram_major": "乾",
            "hexagram_code": [1, 1, 1, 1, 1, 1],
            "flying_star": "九紫",
            "note": format!("{}年运势分析：当前处于己运丑世，天时向好，宜积极进取。", year),
            "lunar": {
                "lunar_year": "乙巳年",
                "lunar_month": "十一月",
                "lunar_day": "初三",
                "ganzhi_year": "乙巳",
                "ganzhi_month": "丁亥",
                "ganzhi_day": "甲子",
                "ganzhi_hour": "甲子",
                "zodiac": "蛇",
                "solar_term": "大雪",
                "twelve_officer": "建",
                "aus_directions": "东南",
                "yi": ["祭祀", "祈福", "出行"],
                "ji": ["动土", "安葬"]
            },
            "period_info": {
                "yuan": {"name": "元", "start_year": -67017, "end_year": 62983, "index": 1, "max_index": 1},
                "hui": {"name": "元会", "start_year": 1744, "end_year": 12543, "index": 1, "max_index": 12},
                "yun": {"name": "己运", "start_year": 1864, "end_year": 2223, "index": 6, "max_index": 12},
                "shi": {"name": "丑世", "start_year": 2014, "end_year": 2043, "index": 2, "max_index": 12},
                "xun": {"name": "甲戌旬", "start_year": 2024, "end_year": 2033, "index": 2, "max_index": 3},
                "year_gua": "乾"
            },
            "next_yun_start_year": 2224,
            "next_shi_start_year": 2044,
            "next_xun_start_year": 2034,
            "mapping_record": {
                "gregorian_year": year,
                "ganzhi": "乙巳",
                "nian_hexagram": "乾",
                "dynasty": "当代",
                "person": "",
                "yuan_raw": "1",
                "hui_raw": "1",
                "yun_raw": "6",
                "shi_raw": "2",
                "xun_raw": "2"
            }
        }
    }))
}

// 获取历史相关事件
async fn get_history_related(Query(params): Query<HistoryRelatedQuery>) -> impl IntoResponse {
    let year = params.year.unwrap_or(2025);
    let _limit = params.limit.unwrap_or(3);
    
    tracing::debug!("📚 获取相关历史: year={}, limit={}", year, _limit);
    
    Json(json!({
        "events": [
            {"year": year - 60, "title": "甲子年事件", "description": "六十年前的重要历史事件"},
            {"year": year - 120, "title": "往年大事", "description": "一百二十年前的历史记载"},
            {"year": year - 180, "title": "古代记录", "description": "一百八十年前的历史文献"}
        ]
    }))
}

// 获取映射记录
async fn get_mapping(Query(params): Query<MappingQuery>) -> impl IntoResponse {
    let year = params.year.unwrap_or(2025);
    
    tracing::debug!("🗺️ 获取映射记录: year={}", year);
    
    Json(json!({
        "record": {
            "year": year,
            "nian_hexagram": "乾",
            "yue_hexagram": "坤",
            "ri_hexagram": "屯",
            "yuan_index": 1,
            "hui_index": 1,
            "yun_index": 6,
            "shi_index": 2
        }
    }))
}

// 获取时间线
async fn get_timeline(Query(params): Query<TimelineQuery>) -> impl IntoResponse {
    // 从 datetime 参数中提取年份
    let year: i32 = params.datetime
        .split('-')
        .next()
        .and_then(|s| s.parse().ok())
        .unwrap_or(2025);
    
    tracing::debug!("📅 查询时间线: {} (from datetime: {})", year, params.datetime);
    
    let data = TIMELINE_DATA.read().unwrap();
    if let Some(timeline) = data.get(&year) {
        Json(timeline.clone())
    } else {
        // 返回完整的模拟数据，完全匹配前端 TimelineData 类型
        Json(json!({
            "year": year,
            "current": {
                "yuan": {
                    "index": 1,
                    "name": "元",
                    "start_year": -67017,
                    "end_year": 62983,
                    "max_index": 1
                },
                "hui": {
                    "index": 1,
                    "name": "元会",
                    "start_year": 1744,
                    "end_year": 12543,
                    "max_index": 12
                },
                "yun": {
                    "index": 6,
                    "name": "己运",
                    "start_year": 1864,
                    "end_year": 2223,
                    "max_index": 12
                },
                "shi": {
                    "index": 2,
                    "name": "丑世",
                    "start_year": 2014,
                    "end_year": 2043,
                    "max_index": 12
                },
                "xun": {
                    "index": 2,
                    "name": "甲戌旬",
                    "start_year": 2024,
                    "end_year": 2033,
                    "max_index": 3
                },
                "year_gua": "乾"
            },
            "yuan_list": [
                {"index": 1, "name": "元", "start_year": -67017, "end_year": 62983, "max_index": 1}
            ],
            "hui_list": [
                {"index": 1, "name": "元会", "start_year": 1744, "end_year": 12543, "max_index": 12}
            ],
            "yun_list": [
                {"index": 5, "name": "戊运", "start_year": 1504, "end_year": 1863, "max_index": 12},
                {"index": 6, "name": "己运", "start_year": 1864, "end_year": 2223, "max_index": 12},
                {"index": 7, "name": "庚运", "start_year": 2224, "end_year": 2583, "max_index": 12}
            ],
            "shi_list": [
                {"index": 1, "name": "子世", "start_year": 1984, "end_year": 2013, "max_index": 12},
                {"index": 2, "name": "丑世", "start_year": 2014, "end_year": 2043, "max_index": 12},
                {"index": 3, "name": "寅世", "start_year": 2044, "end_year": 2073, "max_index": 12}
            ],
            "xun_list": [
                {"index": 1, "name": "甲子旬", "start_year": 2014, "end_year": 2023, "max_index": 3},
                {"index": 2, "name": "甲戌旬", "start_year": 2024, "end_year": 2033, "max_index": 3},
                {"index": 3, "name": "甲申旬", "start_year": 2034, "end_year": 2043, "max_index": 3}
            ]
        }))
    }
}

// 获取历史数据
async fn get_history() -> impl IntoResponse {
    Json(HISTORY_DATA.read().unwrap().clone())
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
