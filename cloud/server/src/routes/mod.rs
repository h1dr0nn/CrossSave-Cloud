pub mod auth;
pub mod device;
pub mod save;

use axum::{
    routing::{get, post},
    Json, Router,
};
use serde_json::{json, Value};
use crate::storage::S3Client;

/// Health check endpoint
pub async fn health() -> Json<Value> {
    Json(json!({
        "ok": true,
        "status": "healthy"
    }))
}

/// Create router with all routes
pub fn create_router(s3_client: S3Client) -> Router {
    Router::new()
        // Health check
        .route("/health", get(health))
        
        // Auth routes (no authentication required)
        .route("/signup", post(auth::handle_signup))
        .route("/login", post(auth::handle_login))
        
        // Device routes (authentication required)
        .route("/device/register", post(device::handle_register_device))
        .route("/device/list", get(device::handle_list_devices))
        .route("/device/remove", post(device::handle_remove_device))
        
        // Save routes (authentication required)
        .route("/save/upload-url", post(save::handle_upload_url))
        .route("/save/upload-content", post(save::handle_upload_content))
        .route("/save/notify-upload", post(save::handle_notify_upload))
        .route("/save/download-url", post(save::handle_download_url))
        .route("/save/list", post(save::handle_list_saves))
        .route("/save/games", post(save::handle_list_games))
        
        // Add S3 client to state
        .with_state(s3_client)
}
