use axum::{http::StatusCode, Json};
use serde::Serialize;
use serde_json::{json, Value};

#[derive(Debug, Serialize)]
pub struct HealthResponse {
    pub status: String,
    pub version: String,
    pub uptime_seconds: u64,
}

// Global start time to calculate uptime
// In a real app, this might be stored in a shared state, but lazy_static or once_cell is fine for simple stats
use std::sync::OnceLock;
use std::time::Instant;

static START_TIME: OnceLock<Instant> = OnceLock::new();

pub fn init_health_check() {
    START_TIME.get_or_init(Instant::now);
}

pub async fn handle_health_check() -> (StatusCode, Json<Value>) {
    let uptime = START_TIME.get().map(|t| t.elapsed().as_secs()).unwrap_or(0);
    
    let response = HealthResponse {
        status: "ok".to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
        uptime_seconds: uptime,
    };

    (StatusCode::OK, Json(json!(response)))
}
