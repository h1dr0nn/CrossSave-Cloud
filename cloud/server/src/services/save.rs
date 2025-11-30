use crate::{
    auth::{jwt::{sign_jwt, verify_jwt}, AuthContext},
    error::AppError,
    routes::save::{
        DownloadUrlResponse, ListGamesResponse, ListSavesRequest, ListSavesResponse,
        NotifyUploadRequest, SaveVersionDto, UploadUrlResponse,
    },
    storage::{get_save_object_key, load_save_metadata, save_save_metadata, S3Client},
    types::{Claims, DownloadPayload, SaveVersion, UploadPayload, WorkerTokenClaims},
    validation::{
        validate_file_list, validate_game_id, validate_sha256, validate_size_bytes,
        validate_version_id,
    },
};
use serde_json::json;

const PRESIGN_TTL_SECONDS: u64 = 300; // 5 minutes
const WORKER_TOKEN_TTL_SECONDS: i64 = 60; // 1 minute

pub struct SaveService;

impl SaveService {
    /// Sign worker token
    fn sign_worker_token(claims: &WorkerTokenClaims) -> Result<String, AppError> {
        // Reuse JWT infrastructure with worker token payload
        let jwt_claims = Claims {
            user_id: serde_json::to_string(claims).map_err(|e| AppError::InternalError(e.into()))?,
            device_id: None,
            exp: claims.exp,
        };

        sign_jwt(&jwt_claims).map_err(|e| AppError::InternalError(e))
    }

    /// Verify worker token
    fn verify_worker_token(token: &str) -> Result<WorkerTokenClaims, AppError> {
        let claims = verify_jwt(token).map_err(|_| AppError::AuthError("invalid_worker_token".to_string()))?;
        let worker_claims: WorkerTokenClaims = serde_json::from_str(&claims.user_id)
            .map_err(|_| AppError::AuthError("invalid_worker_token".to_string()))?;

        Ok(worker_claims)
    }

    pub async fn get_upload_url(
        client: &S3Client,
        auth: &AuthContext,
        payload: UploadPayload,
    ) -> Result<UploadUrlResponse, AppError> {
        // Validate payload
        if !validate_game_id(&payload.game_id) || !validate_version_id(&payload.version_id) {
            return Err(AppError::InvalidInput("invalid_payload".to_string()));
        }

        if !validate_sha256(&payload.sha256) || !validate_size_bytes(payload.size_bytes) {
            return Err(AppError::InvalidInput("invalid_payload".to_string()));
        }

        if !validate_file_list(&payload.file_list) {
            return Err(AppError::InvalidInput("invalid_file_list".to_string()));
        }

        let object_key = get_save_object_key(&auth.user_id, &payload.game_id, &payload.version_id);

        // Generate presigned URL
        let upload_url = client
            .presign_put(&object_key, PRESIGN_TTL_SECONDS)
            .await
            .map_err(|e| AppError::InternalError(e.into()))?;

        // Generate worker token for notify-upload verification
        let now = chrono::Utc::now().timestamp();
        let worker_claims = WorkerTokenClaims {
            user_id: auth.user_id.clone(),
            device_id: auth.device_id.clone(),
            r2_key: object_key.clone(),
            version_id: payload.version_id.clone(),
            exp: now + WORKER_TOKEN_TTL_SECONDS,
        };

        let worker_token = Self::sign_worker_token(&worker_claims)?;

        Ok(UploadUrlResponse {
            ok: true,
            upload_url,
            r2_key: object_key,
            version_id: payload.version_id,
            worker_token,
        })
    }

    pub async fn notify_upload(
        client: &S3Client,
        auth: &AuthContext,
        req: NotifyUploadRequest,
    ) -> Result<serde_json::Value, AppError> {
        // Validate payload
        if !validate_game_id(&req.game_id) || !validate_version_id(&req.version_id) {
            return Err(AppError::InvalidInput("invalid_payload".to_string()));
        }

        if !validate_sha256(&req.sha256) || !validate_size_bytes(req.size_bytes) {
            return Err(AppError::InvalidInput("invalid_payload".to_string()));
        }

        // Verify worker token
        let worker_claims = Self::verify_worker_token(&req.worker_token)?;

        let object_key = get_save_object_key(&auth.user_id, &req.game_id, &req.version_id);

        // Verify token matches request
        if worker_claims.user_id != auth.user_id
            || worker_claims.version_id != req.version_id
            || worker_claims.r2_key != object_key
        {
            return Err(AppError::AuthError("invalid_worker_token".to_string()));
        }

        // Verify upload exists in S3
        let exists = client
            .head_object(&object_key)
            .await
            .map_err(|e| AppError::InternalError(e.into()))?;

        if !exists {
            return Err(AppError::NotFound("upload_missing".to_string()));
        }

        // Load current metadata
        let mut metadata = load_save_metadata(client, &auth.user_id)
            .await
            .map_err(|e| AppError::InternalError(e.into()))?;

        // Create new version entry
        let now = chrono::Utc::now().timestamp();
        let entry = SaveVersion {
            version_id: req.version_id.clone(),
            game_id: req.game_id.clone(),
            size_bytes: req.size_bytes,
            sha256: req.sha256,
            file_list: req.file_list,
            emulator_id: req.emulator_id,
            device_id: req.device_id.or(auth.device_id.clone()),
            timestamp: now,
        };

        // Remove existing version with same ID and prepend new one
        metadata.versions.retain(|v| v.version_id != req.version_id);
        metadata.versions.insert(0, entry);

        // Save metadata
        save_save_metadata(client, &auth.user_id, &metadata)
            .await
            .map_err(|e| AppError::InternalError(e.into()))?;

        Ok(json!({ "ok": true }))
    }

    pub async fn get_download_url(
        client: &S3Client,
        auth: &AuthContext,
        payload: DownloadPayload,
    ) -> Result<DownloadUrlResponse, AppError> {
        // Validate payload
        if !validate_game_id(&payload.game_id) || !validate_version_id(&payload.version_id) {
            return Err(AppError::InvalidInput("invalid_payload".to_string()));
        }

        // Load metadata
        let metadata = load_save_metadata(client, &auth.user_id)
            .await
            .map_err(|e| AppError::InternalError(e.into()))?;

        // Find version
        let version = metadata
            .versions
            .iter()
            .find(|v| v.version_id == payload.version_id && v.game_id == payload.game_id)
            .ok_or_else(|| AppError::NotFound("version_not_found".to_string()))?;

        let object_key = get_save_object_key(&auth.user_id, &payload.game_id, &payload.version_id);

        // Verify object exists
        let exists = client
            .head_object(&object_key)
            .await
            .map_err(|e| AppError::InternalError(e.into()))?;

        if !exists {
            return Err(AppError::NotFound("object_missing".to_string()));
        }

        // Generate presigned URL
        let download_url = client
            .presign_get(&object_key, PRESIGN_TTL_SECONDS)
            .await
            .map_err(|e| AppError::InternalError(e.into()))?;

        Ok(DownloadUrlResponse {
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
        })
    }

    pub async fn list_saves(
        client: &S3Client,
        auth: &AuthContext,
        req: ListSavesRequest,
    ) -> Result<ListSavesResponse, AppError> {
        let game_id = req.game_id.trim();

        // Require minimum 3 characters
        if game_id.len() < 3 {
            return Err(AppError::InvalidInput("game_id_too_short".to_string()));
        }

        // Load metadata
        let metadata = load_save_metadata(client, &auth.user_id)
            .await
            .map_err(|e| AppError::InternalError(e.into()))?;

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

        Ok(ListSavesResponse {
            ok: true,
            game_id: game_id.to_string(),
            versions,
        })
    }

    pub async fn list_games(
        client: &S3Client,
        auth: &AuthContext,
    ) -> Result<ListGamesResponse, AppError> {
        // Load metadata
        let metadata = load_save_metadata(client, &auth.user_id)
            .await
            .map_err(|e| AppError::InternalError(e.into()))?;

        // Extract unique game IDs
        let mut game_ids: Vec<String> = metadata
            .versions
            .iter()
            .map(|v| v.game_id.clone())
            .collect::<std::collections::HashSet<_>>()
            .into_iter()
            .collect();

        game_ids.sort();

        Ok(ListGamesResponse {
            ok: true,
            games: game_ids,
        })
    }
}
