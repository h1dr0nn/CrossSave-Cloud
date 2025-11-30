use axum::{extract::State, http::StatusCode, Json};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

use crate::{
    auth::AuthContext,
    storage::{get_save_object_key, load_save_metadata, save_save_metadata, S3Client},
    types::{DownloadPayload, SaveVersion, UploadPayload, WorkerTokenClaims},
    validation::{
        validate_file_list, validate_game_id, validate_sha256, validate_size_bytes,
        validate_version_id,
    },
};

const PRESIGN_TTL_SECONDS: u64 = 300; // 5 minutes
const WORKER_TOKEN_TTL_SECONDS: i64 = 60; // 1 minute

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

/// Sign worker token
fn sign_worker_token(claims: &WorkerTokenClaims) -> Result<String, anyhow::Error> {
    use crate::auth::jwt::sign_jwt;
    use crate::types::Claims;
    
    // Reuse JWT infrastructure with worker token payload
    let jwt_claims = Claims {
        user_id: serde_json::to_string(claims)?,
        device_id: None,
        exp: claims.exp,
    };
    
    sign_jwt(&jwt_claims)
}

/// Verify worker token
fn verify_worker_token(token: &str) -> Result<WorkerTokenClaims, anyhow::Error> {
    use crate::auth::jwt::verify_jwt;
    
    let claims = verify_jwt(token)?;
    let worker_claims: WorkerTokenClaims = serde_json::from_str(&claims.user_id)?;
    
    Ok(worker_claims)
}

/// Handle upload URL generation
pub async fn handle_upload_url(
    auth: AuthContext,
    State(client): State<S3Client>,
    Json(payload): Json<UploadPayload>,
) -> Result<Json<UploadUrlResponse>, (StatusCode, Json<Value>)> {
    // Validate payload
    if !validate_game_id(&payload.game_id) || !validate_version_id(&payload.version_id) {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(json!({ "error": "invalid_payload" })),
        ));
    }

    if !validate_sha256(&payload.sha256) || !validate_size_bytes(payload.size_bytes) {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(json!({ "error": "invalid_payload" })),
        ));
    }

    if !validate_file_list(&payload.file_list) {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(json!({ "error": "invalid_file_list" })),
        ));
    }

    let object_key = get_save_object_key(&auth.user_id, &payload.game_id, &payload.version_id);

    // Generate presigned URL
    let upload_url = client
        .presign_put(&object_key, PRESIGN_TTL_SECONDS)
        .await
        .map_err(|e| {
            tracing::error!("Failed to generate presigned URL: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": "presign_failed" })),
            )
        })?;

    // Generate worker token for notify-upload verification
    let now = chrono::Utc::now().timestamp();
    let worker_claims = WorkerTokenClaims {
        user_id: auth.user_id.clone(),
        device_id: auth.device_id.clone(),
        r2_key: object_key.clone(),
        version_id: payload.version_id.clone(),
        exp: now + WORKER_TOKEN_TTL_SECONDS,
    };

    let worker_token = sign_worker_token(&worker_claims).map_err(|e| {
        tracing::error!("Failed to sign worker token: {}", e);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "error": "token_sign_failed" })),
        )
    })?;

    Ok(Json(UploadUrlResponse {
        ok: true,
        upload_url,
        r2_key: object_key,
        version_id: payload.version_id,
        worker_token,
    }))
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
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    // Validate payload
    if !validate_game_id(&req.game_id) || !validate_version_id(&req.version_id) {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(json!({ "error": "invalid_payload" })),
        ));
    }

    if !validate_sha256(&req.sha256) || !validate_size_bytes(req.size_bytes) {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(json!({ "error": "invalid_payload" })),
        ));
    }

    // Verify worker token
    let worker_claims = verify_worker_token(&req.worker_token).map_err(|_| {
        (
            StatusCode::UNAUTHORIZED,
            Json(json!({ "error": "invalid_worker_token" })),
        )
    })?;

    let object_key = get_save_object_key(&auth.user_id, &req.game_id, &req.version_id);

    // Verify token matches request
    if worker_claims.user_id != auth.user_id
        || worker_claims.version_id != req.version_id
        || worker_claims.r2_key != object_key
    {
        return Err((
            StatusCode::UNAUTHORIZED,
            Json(json!({ "error": "invalid_worker_token" })),
        ));
    }

    // Verify upload exists in S3
    let exists = client.head_object(&object_key).await.map_err(|e| {
        tracing::error!("Failed to check object: {}", e);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "error": "storage_error" })),
        )
    })?;

    if !exists {
        return Err((
            StatusCode::NOT_FOUND,
            Json(json!({ "error": "upload_missing" })),
        ));
    }

    // Load current metadata
    let mut metadata = load_save_metadata(&client, &auth.user_id)
        .await
        .map_err(|e| {
            tracing::error!("Failed to load metadata: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": "metadata_load_failed" })),
            )
        })?;

    // Create new version entry
    let now = chrono::Utc::now().timestamp();
    let entry = SaveVersion {
        version_id: req.version_id.clone(),
        game_id: req.game_id.clone(),
        size_bytes: req.size_bytes,
        sha256: req.sha256,
        file_list: req.file_list,
        emulator_id: req.emulator_id,
        device_id: req.device_id.or(auth.device_id),
        timestamp: now,
    };

    // Remove existing version with same ID and prepend new one
    metadata.versions.retain(|v| v.version_id != req.version_id);
    metadata.versions.insert(0, entry);

    // Save metadata
    save_save_metadata(&client, &auth.user_id, &metadata)
        .await
        .map_err(|e| {
            tracing::error!("Failed to save metadata: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": "metadata_save_failed" })),
            )
        })?;

    Ok(Json(json!({ "ok": true })))
}

/// Handle download URL generation
pub async fn handle_download_url(
    auth: AuthContext,
    State(client): State<S3Client>,
    Json(payload): Json<DownloadPayload>,
) -> Result<Json<DownloadUrlResponse>, (StatusCode, Json<Value>)> {
    // Validate payload
    if !validate_game_id(&payload.game_id) || !validate_version_id(&payload.version_id) {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(json!({ "error": "invalid_payload" })),
        ));
    }

    // Load metadata
    let metadata = load_save_metadata(&client, &auth.user_id)
        .await
        .map_err(|e| {
            tracing::error!("Failed to load metadata: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": "metadata_load_failed" })),
            )
        })?;

    // Find version
    let version = metadata
        .versions
        .iter()
        .find(|v| v.version_id == payload.version_id && v.game_id == payload.game_id)
        .ok_or_else(|| {
            (
                StatusCode::NOT_FOUND,
                Json(json!({ "error": "version_not_found" })),
            )
        })?;

    let object_key = get_save_object_key(&auth.user_id, &payload.game_id, &payload.version_id);

    // Verify object exists
    let exists = client.head_object(&object_key).await.map_err(|e| {
        tracing::error!("Failed to check object: {}", e);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "error": "storage_error" })),
        )
    })?;

    if !exists {
        return Err((
            StatusCode::NOT_FOUND,
            Json(json!({ "error": "object_missing" })),
        ));
    }

    // Generate presigned URL
    let download_url = client
        .presign_get(&object_key, PRESIGN_TTL_SECONDS)
        .await
        .map_err(|e| {
            tracing::error!("Failed to generate presigned URL: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": "presign_failed" })),
            )
        })?;

    Ok(Json(DownloadUrlResponse {
        ok: true,
        download_url,
        r2_key: object_key,
        version_id: version.version_id.clone(),
        game_id: version.game_id.clone(),
        size_bytes: version.size_bytes,
        sha256: version.sha256.clone(),
        file_list: version.file_list.clone(),
        emulator_id: version.emulator_id.clone(),
        timestamp: version.timestamp,
    }))
}

/// Handle list saves
pub async fn handle_list_saves(
    auth: AuthContext,
    State(client): State<S3Client>,
    Json(req): Json<ListSavesRequest>,
) -> Result<Json<ListSavesResponse>, (StatusCode, Json<Value>)> {
    let game_id = req.game_id.trim();

    // Require minimum 3 characters
    if game_id.len() < 3 {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(json!({ "error": "game_id_too_short" })),
        ));
    }

    // Load metadata
    let metadata = load_save_metadata(&client, &auth.user_id)
        .await
        .map_err(|e| {
            tracing::error!("Failed to load metadata: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": "metadata_load_failed" })),
            )
        })?;

    // Filter by partial game ID match (case-insensitive)
    let search_lower = game_id.to_lowercase();
    let mut versions: Vec<SaveVersionDto> = metadata
        .versions
        .iter()
        .filter(|v| v.game_id.to_lowercase().contains(&search_lower))
        .map(|v| SaveVersionDto {
            version_id: v.version_id.clone(),
            game_id: v.game_id.clone(),
            size_bytes: v.size_bytes,
            timestamp: v.timestamp,
            device_id: v.device_id.clone(),
            sha256: v.sha256.clone(),
            file_list: v.file_list.clone(),
        })
        .collect();

    // Sort by timestamp descending
    versions.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));

    Ok(Json(ListSavesResponse {
        ok: true,
        game_id: game_id.to_string(),
        versions,
    }))
}

/// Handle list games
pub async fn handle_list_games(
    auth: AuthContext,
    State(client): State<S3Client>,
) -> Result<Json<ListGamesResponse>, (StatusCode, Json<Value>)> {
    // Load metadata
    let metadata = load_save_metadata(&client, &auth.user_id)
        .await
        .map_err(|e| {
            tracing::error!("Failed to load metadata: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": "metadata_load_failed" })),
            )
        })?;

    // Extract unique game IDs
    let mut game_ids: Vec<String> = metadata
        .versions
        .iter()
        .map(|v| v.game_id.clone())
        .collect::<std::collections::HashSet<_>>()
        .into_iter()
        .collect();

    game_ids.sort();

    Ok(Json(ListGamesResponse {
        ok: true,
        games: game_ids,
    }))
}
