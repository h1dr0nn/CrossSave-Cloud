use axum::{extract::State, http::StatusCode, Json};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

use crate::{
    auth::AuthContext,
    error::AppError,
    services::save::SaveService,
    storage::S3Client,
    types::{DownloadPayload, UploadPayload},
};

#[derive(Debug, Serialize)]
pub struct UploadUrlResponse {
    pub ok: bool,
    pub upload_url: String,
    pub r2_key: String,
    pub version_id: String,
    pub worker_token: String,
}

#[derive(Debug, Serialize)]
pub struct DownloadUrlResponse {
    pub ok: bool,
    pub download_url: String,
    pub r2_key: String,
    pub version_id: String,
    pub game_id: String,
    pub size_bytes: u64,
    pub sha256: String,
    pub file_list: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub emulator_id: Option<String>,
    pub timestamp: i64,
}

#[derive(Debug, Serialize)]
pub struct ListSavesResponse {
    pub ok: bool,
    pub game_id: String,
    pub versions: Vec<SaveVersionDto>,
}

#[derive(Debug, Serialize)]
pub struct SaveVersionDto {
    pub version_id: String,
    pub game_id: String,
    pub size_bytes: u64,
    pub timestamp: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub device_id: Option<String>,
    pub sha256: String,
    pub file_list: Vec<String>,
}

#[derive(Debug, Serialize)]
pub struct ListGamesResponse {
    pub ok: bool,
    pub games: Vec<String>,
}

#[derive(Debug, Deserialize)]
pub struct ListSavesRequest {
    pub game_id: String,
}

#[derive(Debug, Deserialize)]
pub struct NotifyUploadRequest {
    pub game_id: String,
    pub version_id: String,
    pub size_bytes: u64,
    pub sha256: String,
    pub file_list: Vec<String>,
    #[serde(default)]
    pub emulator_id: Option<String>,
    #[serde(default)]
    pub device_id: Option<String>,
    pub worker_token: String,
}

/// Handle upload URL generation
pub async fn handle_upload_url(
    auth: AuthContext,
    State(client): State<S3Client>,
    Json(payload): Json<UploadPayload>,
) -> Result<Json<UploadUrlResponse>, AppError> {
    let response = SaveService::get_upload_url(&client, &auth, payload).await?;
    Ok(Json(response))
}

/// Handle upload content (direct upload proxy - not used with presigned URLs)
pub async fn handle_upload_content(
    State(_client): State<S3Client>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    // In this implementation, we use presigned URLs directly
    // This endpoint is kept for API compatibility but returns not implemented
    Err((
        StatusCode::NOT_IMPLEMENTED,
        Json(json!({ "error": "use_presigned_url" })),
    ))
}

/// Handle notify upload
pub async fn handle_notify_upload(
    auth: AuthContext,
    State(client): State<S3Client>,
    Json(req): Json<NotifyUploadRequest>,
) -> Result<Json<Value>, AppError> {
    let response = SaveService::notify_upload(&client, &auth, req).await?;
    Ok(Json(response))
}

/// Handle download URL generation
pub async fn handle_download_url(
    auth: AuthContext,
    State(client): State<S3Client>,
    Json(payload): Json<DownloadPayload>,
) -> Result<Json<DownloadUrlResponse>, AppError> {
    let response = SaveService::get_download_url(&client, &auth, payload).await?;
    Ok(Json(response))
}

/// Handle list saves
pub async fn handle_list_saves(
    auth: AuthContext,
    State(client): State<S3Client>,
    Json(req): Json<ListSavesRequest>,
) -> Result<Json<ListSavesResponse>, AppError> {
    let response = SaveService::list_saves(&client, &auth, req).await?;
    Ok(Json(response))
}

/// Handle list games
pub async fn handle_list_games(
    auth: AuthContext,
    State(client): State<S3Client>,
) -> Result<Json<ListGamesResponse>, AppError> {
    let response = SaveService::list_games(&client, &auth).await?;
    Ok(Json(response))
}
