use axum::{
    routing::{get, post},
    Json, Router, extract::{Query, Path},
};
use axum::response::IntoResponse;
use axum::http::HeaderValue;
use chrono::{DateTime, Utc, Datelike};
use serde::{Deserialize, Serialize};
use tower_http::cors::CorsLayer;
use std::sync::RwLock;
use std::collections::HashMap;
use std::env;
use reqwest::Client;
use tokio::task;
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
        
        // API路由
        .route("/api/calculate", post(calculate))
        .route("/api/timeline/:year", get(get_timeline))
        .route("/api/history", get(get_history))
        .route("/api/celestial/hashes", get(get_celestial_hashes))
        .route("/api/sky/settings", get(get_sky_settings))
        .route("/api/sky/settings", post(update_sky_settings))
        
        // 静态文件服务
        .route("/static/:file", get(static_handler))
        
        // CORS
        .layer(
            CorsLayer::new()
                .allow_origin(HeaderValue::from_str("https://huangji-jingshi.vercel.app").unwrap())
                .allow_methods(tower_http::cors::Any)
                .allow_headers(tower_http::cors::Any),
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

    // 加载时间线数据
    let timeline_path = data_path.parent().unwrap().parent().unwrap().join("data/history.json");
    if timeline_path.exists() {
        match load_json_file(&timeline_path).await {
            Ok(data) => {
                *TIMELINE_DATA.write().unwrap() = data.as_object().cloned().unwrap_or_default();
                tracing::info!("✅ 时间线数据加载成功");
            }
            Err(e) => tracing::warn!("⚠️ 时间线数据加载失败: {}", e),
        }
    }

    // 加载历史数据
    let history_path = data_path.parent().unwrap().join("major_events.json");
    if history_path.exists() {
        match load_json_file(&history_path).await {
            Ok(data) => {
                *HISTORY_DATA.write().unwrap() = data;
                tracing::info!("✅ 历史数据加载成功");
            }
            Err(e) => tracing::warn!("⚠️ 历史数据加载失败: {}", e),
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
        "version": "1.0.0-fixed",
        "message": "API服务正常运行",
        "endpoints": [
            "GET /health",
            "POST /api/calculate",
            "GET /api/timeline/{year}",
            "GET /api/history",
            "GET /api/celestial/hashes",
            "GET /api/sky/settings",
            "POST /api/sky/settings"
        ]
    }))
}

// 天机演算
async fn calculate(Json(payload): Json<serde_json::Value>) -> impl IntoResponse {
    tracing::info!("🔮 收到演算请求: {:?}", payload);

    // 模拟演算过程
    tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;

    Json(json!({
        "result": "天机演算完成",
        "calculation_id": "calc_" + &Utc::now().timestamp().to_string(),
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

// 获取时间线
async fn get_timeline(Path(year): Path<i32>) -> impl IntoResponse {
    tracing::debug!("📅 查询时间线: {}", year);
    
    let data = TIMELINE_DATA.read().unwrap();
    if let Some(timeline) = data.get(&year) {
        Json(timeline.clone())
    } else {
        Json(json!({
            "year": year,
            "ganzhi": "甲子年",
            "events": []
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
