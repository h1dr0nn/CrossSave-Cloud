use std::fs;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;

use async_trait::async_trait;
use reqwest::{
    header::{HeaderMap, HeaderValue, CONTENT_LENGTH, CONTENT_TYPE},
    Client,
};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use thiserror::Error;
use tokio::io::AsyncWriteExt;
use tracing::{debug, error, info, warn};
use uuid::Uuid;

use crate::core::packager::SaveMetadata;
use crate::core::settings::{CloudMode, SettingsManager};

// =============================================================================
// HELPERS
// =============================================================================

/// Sanitize game_id to match backend validation: /^[A-Za-z0-9_.-]{1,128}$/
/// Replaces spaces and other invalid characters with underscores
fn sanitize_game_id(game_id: &str) -> String {
    game_id
        .chars()
        .map(|c| {
            if c.is_alphanumeric() || c == '_' || c == '.' || c == '-' {
                c
            } else {
                '_' // Replace invalid chars (including spaces) with underscore
            }
        })
        .collect::<String>()
        .chars()
        .take(128) // Limit to 128 chars
        .collect()
}

// =============================================================================
// Configuration
// =============================================================================

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CloudConfig {
    pub enabled: bool,
    pub base_url: String,
    pub api_key: String,
    pub device_id: String,
    pub device_name: String,
    pub platform: String,
    pub user_id: String,
    pub timeout_seconds: u64,
    pub has_registered_device: bool,
}

impl Default for CloudConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            base_url: "https://crosssave-official-cloud.hdrn151.workers.dev".to_string(),
            api_key: String::new(),
            device_id: String::new(),
            device_name: String::new(),
            platform: String::new(),
            user_id: String::new(),
            timeout_seconds: 30,
            has_registered_device: false,
        }
    }
}

// =============================================================================
// Cloud Version Summary
// =============================================================================

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CloudVersionSummary {
    pub version_id: String,
    pub timestamp: u64,
    pub size_bytes: u64,
    pub device_id: String,
    pub file_list: Vec<String>,
    pub sha256: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CloudDevice {
    pub device_id: String,
    pub platform: String,
    pub device_name: String,
    pub last_seen: u64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct UploadRequest {
    pub game_id: String,
    pub version_id: String,
    pub size_bytes: u64,
    pub sha256: String,
    pub file_list: Vec<String>,
    pub emulator_id: Option<String>,
    pub device_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub worker_token: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct UploadUrlResponse {
    pub upload_url: String,
    pub r2_key: String,
    pub version_id: String,
    #[serde(default)]
    pub worker_token: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DownloadUrlResponse {
    pub ok: bool,
    pub download_url: String,
    pub r2_key: String,
    pub version_id: String,
    pub game_id: String,
    pub size_bytes: u64,
    pub sha256: String,
    #[serde(default)]
    pub file_list: Vec<String>,
    #[serde(default)]
    pub emulator_id: Option<String>,
    #[serde(default)]
    pub timestamp: Option<u64>,
}

// =============================================================================
// Error Types
// =============================================================================

#[derive(Debug, Error)]
#[allow(dead_code)]
pub enum CloudError {
    #[error("cloud sync is not enabled")]
    NotEnabled,
    #[error("cloud backend is disabled")]
    Disabled,
    #[error("network error: {0}")]
    NetworkError(String),
    #[error("storage error: {0}")]
    StorageError(String),
    #[error("not found: {0}")]
    NotFound(String),
    #[error("invalid configuration: {0}")]
    InvalidConfig(String),
    #[error("io error: {0}")]
    Io(String),
    #[error("serialization error: {0}")]
    Serialization(String),
    #[error("unauthorized: {0}")]
    Unauthorized(String),
}

// =============================================================================
// Cloud Backend Trait
// =============================================================================

#[async_trait]
pub trait CloudBackend: Send + Sync {
    async fn login(&self, email: String, password: String) -> Result<String, CloudError>;
    async fn signup(&self, email: String, password: String) -> Result<String, CloudError>;
    async fn upload_archive(
        &self,
        metadata: SaveMetadata,
        archive_path: PathBuf,
    ) -> Result<CloudVersionSummary, CloudError>;
    async fn request_upload_url(
        &self,
        payload: UploadRequest,
    ) -> Result<UploadUrlResponse, CloudError>;
    async fn notify_upload_complete(&self, payload: UploadRequest) -> Result<(), CloudError>;
    async fn request_download_url(
        &self,
        game_id: String,
        version_id: String,
    ) -> Result<DownloadUrlResponse, CloudError>;
    async fn list_versions(
        &self,
        game_id: String,
        limit: Option<usize>,
    ) -> Result<Vec<CloudVersionSummary>, CloudError>;
    async fn download_version(
        &self,
        game_id: String,
        version_id: String,
        target_path: PathBuf,
    ) -> Result<(), CloudError>;
    fn ensure_device_id(&self) -> Result<String, CloudError>;
    async fn list_devices(&self, token: String) -> Result<Vec<CloudDevice>, CloudError>;
    async fn register_device(
        &self,
        token: String,
        device_id: String,
        platform: String,
        device_name: String,
    ) -> Result<(), CloudError>;
    async fn remove_device(&self, token: String, device_id: String) -> Result<(), CloudError>;
    fn get_device_id(&self) -> Result<String, CloudError>;
    async fn list_games(&self) -> Result<Vec<String>, CloudError>;
    
    /// Check if the cloud backend is reachable and healthy
    async fn check_connection(&self) -> Result<bool, CloudError>;
}

// =============================================================================
// Disabled Cloud Backend
// =============================================================================

#[derive(Clone, Debug)]
pub struct DisabledCloudBackend;

#[async_trait]
impl CloudBackend for DisabledCloudBackend {
    async fn login(&self, _email: String, _password: String) -> Result<String, CloudError> {
        Err(CloudError::Disabled)
    }

    async fn signup(&self, _email: String, _password: String) -> Result<String, CloudError> {
        Err(CloudError::Disabled)
    }

    async fn upload_archive(
        &self,
        _metadata: SaveMetadata,
        _archive_path: PathBuf,
    ) -> Result<CloudVersionSummary, CloudError> {
        Err(CloudError::Disabled)
    }

    async fn request_upload_url(
        &self,
        _payload: UploadRequest,
    ) -> Result<UploadUrlResponse, CloudError> {
        Err(CloudError::Disabled)
    }

    async fn notify_upload_complete(&self, _payload: UploadRequest) -> Result<(), CloudError> {
        Err(CloudError::Disabled)
    }

    async fn request_download_url(
        &self,
        _game_id: String,
        _version_id: String,
    ) -> Result<DownloadUrlResponse, CloudError> {
        Err(CloudError::Disabled)
    }

    async fn list_versions(
        &self,
        _game_id: String,
        _limit: Option<usize>,
    ) -> Result<Vec<CloudVersionSummary>, CloudError> {
        Err(CloudError::Disabled)
    }

    async fn download_version(
        &self,
        _game_id: String,
        _version_id: String,
        _target_path: PathBuf,
    ) -> Result<(), CloudError> {
        Err(CloudError::Disabled)
    }

    fn ensure_device_id(&self) -> Result<String, CloudError> {
        Err(CloudError::Disabled)
    }

    async fn list_devices(&self, _token: String) -> Result<Vec<CloudDevice>, CloudError> {
        Err(CloudError::Disabled)
    }

    async fn register_device(
        &self,
        _token: String,
        _device_id: String,
        _platform: String,
        _device_name: String,
    ) -> Result<(), CloudError> {
        Err(CloudError::Disabled)
    }

    async fn remove_device(&self, _token: String, _device_id: String) -> Result<(), CloudError> {
        Err(CloudError::Disabled)
    }

    fn get_device_id(&self) -> Result<String, CloudError> {
        Err(CloudError::Disabled)
    }
    
    async fn check_connection(&self) -> Result<bool, CloudError> {
        Ok(false) // Disabled backend is never connected
    }

    async fn list_games(&self) -> Result<Vec<String>, CloudError> {
        Err(CloudError::Disabled)
    }
}

// =============================================================================
// HTTP Cloud Backend
// =============================================================================

#[derive(Clone)]
pub struct HttpCloudBackend {
    client: Client,
    settings: Arc<SettingsManager>,
    mode: CloudMode,
    log_tag: &'static str,
    access_headers: HeaderMap,
}

pub type SelfHostHttpBackend = HttpCloudBackend;

pub fn ensure_device_identity(
    settings: &Arc<SettingsManager>,
) -> Result<(String, String, String), CloudError> {
    let mut app_settings = settings
        .get_settings()
        .map_err(|e| CloudError::InvalidConfig(format!("settings load failed: {e}")))?;

    let mut updated = false;
    if app_settings.cloud.device_id.trim().is_empty() {
        app_settings.cloud.device_id = default_device_id();
        updated = true;
    }

    if app_settings.cloud.platform.trim().is_empty() {
        app_settings.cloud.platform = std::env::consts::OS.to_string();
        updated = true;
    }

    if app_settings.cloud.device_name.trim().is_empty() {
        app_settings.cloud.device_name = default_device_name(&app_settings.cloud.platform);
        updated = true;
    }

    if updated {
        settings
            .update_settings(app_settings.clone())
            .map_err(|e| CloudError::InvalidConfig(format!("settings save failed: {e}")))?;
    }

    Ok((
        app_settings.cloud.device_id,
        app_settings.cloud.platform,
        app_settings.cloud.device_name,
    ))
}

#[derive(Deserialize)]
struct LoginResponse {
    token: String,
    device_id: Option<String>,
    user_id: Option<String>,
}

#[derive(Deserialize)]
struct SignupResponse {
    token: String,
    user_id: String,
    device_id: Option<String>,
}

#[derive(Deserialize)]
struct ListDevicesResponse {
    devices: Vec<CloudDevice>,
}

impl HttpCloudBackend {
    fn collect_access_headers(mode: CloudMode) -> HeaderMap {
        let mut headers = HeaderMap::new();
        if mode != CloudMode::Official {
            return headers;
        }

        let pairs = [
            ("CF_ACCESS_CLIENT_ID", "Cf-Access-Client-Id"),
            ("CF_ACCESS_CLIENT_SECRET", "Cf-Access-Client-Secret"),
            ("CF_ACCESS_JWT_ASSERTION", "Cf-Access-Jwt-Assertion"),
        ];

        for (env_key, header_name) in pairs {
            if let Ok(value) = std::env::var(env_key) {
                let trimmed = value.trim();
                if !trimmed.is_empty() {
                    if let Ok(header_value) = HeaderValue::from_str(trimmed) {
                        headers.insert(header_name, header_value);
                    }
                }
            }
        }

        headers
    }

    fn apply_access_headers(&self, builder: reqwest::RequestBuilder) -> reqwest::RequestBuilder {
        self.access_headers
            .iter()
            .fold(builder, |acc, (name, value)| acc.header(name, value))
    }

    pub fn new(settings: Arc<SettingsManager>, mode: CloudMode) -> Result<Self, CloudError> {
        let config = settings
            .get_settings()
            .map_err(|e| CloudError::InvalidConfig(format!("settings load failed: {e}")))?;
        let timeout = config.cloud.timeout_seconds.max(1);
        let client = Client::builder()
            .timeout(std::time::Duration::from_secs(timeout))
            .build()
            .map_err(|e| CloudError::InvalidConfig(format!("client build failed: {e}")))?;

        let log_tag = log_tag(&mode);
        let access_headers = Self::collect_access_headers(mode.clone());

        Ok(Self {
            client,
            settings,
            mode,
            log_tag,
            access_headers,
        })
    }

    fn validate_base_url(&self) -> Result<String, CloudError> {
        let settings = self
            .settings
            .get_settings()
            .map_err(|e| CloudError::InvalidConfig(format!("settings load failed: {e}")))?;

        let base_url = match self.mode {
            CloudMode::Official => settings.cloud.base_url,
            CloudMode::SelfHost => settings.self_host.api_server,
            CloudMode::Off => return Err(CloudError::Disabled),
        };

        if base_url.trim().is_empty() {
            return Err(CloudError::InvalidConfig("base_url missing".into()));
        }
        Ok(base_url.trim_end_matches('/').to_string())
    }

    fn get_auth_header(&self) -> Result<String, CloudError> {
        let settings = self
            .settings
            .get_settings()
            .map_err(|e| CloudError::InvalidConfig(format!("settings load failed: {e}")))?;

        match self.mode {
            CloudMode::Official => {
                if settings.cloud.api_key.trim().is_empty() {
                    return Err(CloudError::Unauthorized("api_key missing".into()));
                }
                Ok(format!("Bearer {}", settings.cloud.api_key))
            }
            CloudMode::SelfHost => {
                if settings.self_host.access_key.trim().is_empty() {
                    return Err(CloudError::Unauthorized("access_key missing".into()));
                }
                Ok(format!("Bearer {}", settings.self_host.access_key))
            }
            CloudMode::Off => Err(CloudError::Disabled),
        }
    }

    fn ensure_local_device_identity(&self) -> Result<(String, String, String), CloudError> {
        ensure_device_identity(&self.settings)
    }

    async fn ensure_device_registered(&self) -> Result<String, CloudError> {
        let (device_id, platform, device_name) = self.ensure_local_device_identity()?;
        let base_url = self.validate_base_url()?;
        let auth = self.get_auth_header()?;
        let mut last_status: Option<reqwest::StatusCode> = None;
        for attempt in 0..2 {
            let resp = self
                .apply_access_headers(
                    self.client
                        .post(format!("{}/device/register", base_url))
                        .header("Authorization", auth.clone())
                        .json(&serde_json::json!({
                            "device_id": device_id,
                            "platform": platform,
                            "device_name": device_name,
                        })),
                )
                .send()
                .await
                .map_err(|e| CloudError::NetworkError(e.to_string()))?;

            if resp.status() == reqwest::StatusCode::UNAUTHORIZED {
                return Err(CloudError::Unauthorized("invalid token".into()));
            }

            if resp.status().is_success() {
                let mut app_settings = self
                    .settings
                    .get_settings()
                    .map_err(|e| CloudError::InvalidConfig(format!("settings load failed: {e}")))?;
                if !app_settings.cloud.has_registered_device {
                    app_settings.cloud.has_registered_device = true;
                    let _ = self.settings.update_settings(app_settings);
                }

                return Ok(device_id);
            }

            last_status = Some(resp.status());
            error!(
                "{} device register failed (attempt {}): {}",
                self.log_tag,
                attempt + 1,
                resp.status()
            );
        }

        Err(CloudError::NetworkError(format!(
            "device register failed: {}",
            last_status
                .map(|s| s.to_string())
                .unwrap_or_else(|| "unknown status".to_string())
        )))
    }
}

#[async_trait]
impl CloudBackend for HttpCloudBackend {
    async fn signup(&self, email: String, password: String) -> Result<String, CloudError> {
        info!("{} Attempting signup", self.log_tag);
        let base_url = self.validate_base_url()?;
        let (device_id, platform, device_name) = self.ensure_local_device_identity()?;
        let resp = self
            .apply_access_headers(
                self.client
                    .post(format!("{}/signup", base_url))
                    .json(&serde_json::json!({
                        "email": email,
                        "password": password,
                        "device_id": device_id,
                        "platform": platform,
                        "device_name": device_name,
                    })),
            )
            .send()
            .await
            .map_err(|e| CloudError::NetworkError(e.to_string()))?;

        if !resp.status().is_success() {
            let status = resp.status();
            warn!(
                "{} Signup failed with status {}",
                self.log_tag,
                status
            );
            
            // Return user-friendly error messages based on status code
            let error_message = match status.as_u16() {
                409 => "This email address is already registered".to_string(),
                400 => "Invalid email or password format".to_string(),
                401 => "Authentication failed".to_string(),
                403 => "Access denied".to_string(),
                500..=599 => "Server error. Please try again later".to_string(),
                _ => format!("Signup failed. Please try again"),
            };
            
            return Err(CloudError::Unauthorized(error_message));
        }

        let parsed: SignupResponse = resp
            .json()
            .await
            .map_err(|e| CloudError::Serialization(e.to_string()))?;

        let mut app_settings = self
            .settings
            .get_settings()
            .map_err(|e| CloudError::InvalidConfig(format!("settings load failed: {e}")))?;
        app_settings.cloud.enabled = true;
        app_settings.cloud.api_key = parsed.token.clone();
        app_settings.cloud.device_id = parsed.device_id.clone().unwrap_or(device_id);
        if app_settings.cloud.platform.trim().is_empty() {
            app_settings.cloud.platform = platform;
        }
        if app_settings.cloud.device_name.trim().is_empty() {
            app_settings.cloud.device_name = device_name;
        }
        app_settings.cloud.user_id = parsed.user_id.clone();
        app_settings.cloud.has_registered_device = true;
        self.settings
            .update_settings(app_settings)
            .map_err(|e| CloudError::InvalidConfig(format!("settings save failed: {e}")))?;

        Ok(parsed.token)
    }
    async fn login(&self, email: String, password: String) -> Result<String, CloudError> {
        info!("{} Attempting login", self.log_tag);
        let base_url = self.validate_base_url()?;
        let (device_id, platform, device_name) = self.ensure_local_device_identity()?;
        let resp = self
            .apply_access_headers(
                self.client
                    .post(format!("{}/login", base_url))
                    .json(&serde_json::json!({
                        "email": email,
                        "password": password,
                        "device_id": device_id,
                        "platform": platform,
                        "device_name": device_name,
                    })),
            )
            .send()
            .await
            .map_err(|e| CloudError::NetworkError(e.to_string()))?;

        if !resp.status().is_success() {
            let status = resp.status();
            warn!(
                "{} Login failed with status {}",
                self.log_tag,
                status
            );
            
            // Return user-friendly error messages based on status code
            let error_message = match status.as_u16() {
                401 => "Invalid email or password".to_string(),
                404 => "Account not found".to_string(),
                403 => "Access denied".to_string(),
                429 => "Too many login attempts. Please try again later".to_string(),
                500..=599 => "Server error. Please try again later".to_string(),
                _ => "Login failed. Please try again".to_string(),
            };
            
            return Err(CloudError::Unauthorized(error_message));
        }

        let parsed: LoginResponse = resp
            .json()
            .await
            .map_err(|e| CloudError::Serialization(e.to_string()))?;

        let mut app_settings = self
            .settings
            .get_settings()
            .map_err(|e| CloudError::InvalidConfig(format!("settings load failed: {e}")))?;
        app_settings.cloud.enabled = true;
        app_settings.cloud.api_key = parsed.token.clone();
        if let Some(device) = parsed.device_id.clone() {
            app_settings.cloud.device_id = device;
        } else {
            app_settings.cloud.device_id = device_id;
        }
        if app_settings.cloud.platform.trim().is_empty() {
            app_settings.cloud.platform = platform;
        }
        if app_settings.cloud.device_name.trim().is_empty() {
            app_settings.cloud.device_name = device_name;
        }
        if let Some(user_id) = parsed.user_id.clone() {
            app_settings.cloud.user_id = user_id;
        }
        self.settings
            .update_settings(app_settings)
            .map_err(|e| CloudError::InvalidConfig(format!("settings save failed: {e}")))?;

        Ok(parsed.token)
    }
    async fn upload_archive(
        &self,
        metadata: SaveMetadata,
        archive_path: PathBuf,
    ) -> Result<CloudVersionSummary, CloudError> {
        if !archive_path.exists() {
            warn!(
                "{} Archive not found at {}",
                self.log_tag,
                archive_path.display()
            );
            return Err(CloudError::NotFound(format!(
                "archive not found: {}",
                archive_path.display()
            )));
        }

        let device_id = self.ensure_device_registered().await.map_err(|err| {
            warn!("{} failed to register device: {}", self.log_tag, err);
            err
        })?;

        let archive_size = fs::metadata(&archive_path)
            .map_err(|e| CloudError::Io(e.to_string()))?
            .len();
        let hash = calculate_sha256(&archive_path)?;

        let mut upload_request = UploadRequest {
            game_id: metadata.game_id.clone(),
            version_id: metadata.version_id.clone(),
            size_bytes: archive_size,
            sha256: hash.clone(),
            file_list: metadata.file_list.clone(),
            emulator_id: Some(metadata.emulator_id.clone()),
            device_id: Some(device_id.clone()),
            worker_token: None,
        };

        let signed = self.request_upload_url(upload_request.clone()).await?;

        upload_request.worker_token = signed.worker_token.clone();

        let archive_file = fs::read(&archive_path).map_err(|e| CloudError::Io(e.to_string()))?;
        let resp = self
            .client
            .put(&signed.upload_url)
            .header(CONTENT_TYPE, "application/zip")
            .header(CONTENT_LENGTH, archive_size)
            .body(archive_file)
            .send()
            .await
            .map_err(|e| CloudError::NetworkError(e.to_string()))?;

        if !resp.status().is_success() {
            error!(
                "{} Upload failed with status {}",
                self.log_tag,
                resp.status()
            );
            return Err(CloudError::NetworkError(format!(
                "upload failed: {}",
                resp.status()
            )));
        }

        self.notify_upload_complete(upload_request.clone()).await?;

        Ok(CloudVersionSummary {
            version_id: upload_request.version_id,
            timestamp: metadata.timestamp,
            size_bytes: upload_request.size_bytes,
            device_id,
            file_list: upload_request.file_list,
            sha256: hash,
        })
    }

    async fn request_upload_url(
        &self,
        payload: UploadRequest,
    ) -> Result<UploadUrlResponse, CloudError> {
        let device_id = self.ensure_device_registered().await?;
        let mut payload = payload;
        payload.device_id = Some(device_id);
        
        let base_url = self.validate_base_url()?;
        let auth = self.get_auth_header()?;

        let resp = self
            .apply_access_headers(
                self.client
                    .post(format!("{}/save/upload-url", base_url))
                    .header("Authorization", auth)
                    .json(&payload),
            )
            .send()
            .await
            .map_err(|e| CloudError::NetworkError(e.to_string()))?;

        if resp.status() == reqwest::StatusCode::UNAUTHORIZED {
            return Err(CloudError::Unauthorized("invalid token".into()));
        }

        if !resp.status().is_success() {
            return Err(CloudError::NetworkError(format!(
                "upload url failed: {}",
                resp.status()
            )));
        }

        let parsed: UploadUrlResponse = resp
            .json()
            .await
            .map_err(|e| CloudError::Serialization(e.to_string()))?;

        Ok(parsed)
    }

    async fn notify_upload_complete(&self, payload: UploadRequest) -> Result<(), CloudError> {
        let device_id = self.ensure_device_registered().await?;
        let mut payload = payload;
        payload.device_id = Some(device_id);

        let base_url = self.validate_base_url()?;
        let auth = self.get_auth_header()?;

        let resp = self
            .apply_access_headers(
                self.client
                    .post(format!("{}/save/notify-upload", base_url))
                    .header("Authorization", auth)
                    .json(&payload),
            )
            .send()
            .await
            .map_err(|e| CloudError::NetworkError(e.to_string()))?;

        if resp.status() == reqwest::StatusCode::UNAUTHORIZED {
            return Err(CloudError::Unauthorized("invalid token".into()));
        }

        if !resp.status().is_success() {
            return Err(CloudError::NetworkError(format!(
                "notify upload failed: {}",
                resp.status()
            )));
        }

        Ok(())
    }

    async fn request_download_url(
        &self,
        game_id: String,
        version_id: String,
    ) -> Result<DownloadUrlResponse, CloudError> {
        self.ensure_device_registered().await?;
        let base_url = self.validate_base_url()?;
        let auth = self.get_auth_header()?;

        let resp = self
            .apply_access_headers(
                self.client
                    .post(format!("{}/save/download-url", base_url))
                    .header("Authorization", auth)
                    .json(&serde_json::json!({ "game_id": game_id, "version_id": version_id })),
            )
            .send()
            .await
            .map_err(|e| CloudError::NetworkError(e.to_string()))?;

        if resp.status() == reqwest::StatusCode::UNAUTHORIZED {
            return Err(CloudError::Unauthorized("invalid token".into()));
        }

        if !resp.status().is_success() {
            return Err(CloudError::NetworkError(format!(
                "download url failed: {}",
                resp.status()
            )));
        }

        resp.json()
            .await
            .map_err(|e| CloudError::Serialization(e.to_string()))
    }

    async fn list_versions(
        &self,
        game_id: String,
        limit: Option<usize>,
    ) -> Result<Vec<CloudVersionSummary>, CloudError> {
        let base_url = self.validate_base_url()?;
        let auth = self.get_auth_header()?;

        let payload = serde_json::json!({ "game_id": game_id.clone() });

        let resp = self
            .apply_access_headers(
                self.client
                    .post(format!("{}/save/list", base_url))
                    .header("Authorization", auth)
                    .json(&payload),
            )
            .send()
            .await
            .map_err(|e| CloudError::NetworkError(e.to_string()))?;

        if resp.status() == reqwest::StatusCode::UNAUTHORIZED {
            return Err(CloudError::Unauthorized("invalid token".into()));
        }

        if !resp.status().is_success() {
            let status = resp.status();
            let error_body = resp.text().await.unwrap_or_else(|_| "unable to read error body".to_string());
            error!(
                "{} list_versions failed: status={}, game_id={}, error={}",
                self.log_tag, status, game_id, error_body
            );
            return Err(CloudError::NetworkError(format!(
                "list failed: {} - {}",
                status, error_body
            )));
        }

        #[derive(Deserialize)]
        struct SaveListVersion {
            version_id: String,
            size_bytes: u64,
            timestamp: u64,
            #[serde(default)]
            device_id: String,
            sha256: String,
            #[serde(default)]
            file_list: Vec<String>,
        }

        #[derive(Deserialize)]
        struct SaveListResponse {
            ok: bool,
            #[serde(default)]
            versions: Vec<SaveListVersion>,
            #[serde(default)]
            error: Option<String>,
        }

        let parsed: SaveListResponse = resp
            .json()
            .await
            .map_err(|e| CloudError::Serialization(e.to_string()))?;

        if !parsed.ok {
            let message = parsed.error.unwrap_or_else(|| "list_failed".to_string());
            error!("{} list_versions failed: {}", self.log_tag, message);
            return Err(CloudError::NetworkError(message));
        }

        let mut versions: Vec<CloudVersionSummary> = parsed
            .versions
            .into_iter()
            .map(|entry| CloudVersionSummary {
                version_id: entry.version_id,
                timestamp: entry.timestamp,
                size_bytes: entry.size_bytes,
                device_id: entry.device_id,
                file_list: entry.file_list,
                sha256: entry.sha256,
            })
            .collect();

        versions.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));
        if let Some(limit) = limit {
            versions.truncate(limit.min(versions.len()));
        }

        info!(
            "{} list_versions game_id={} count={}",
            self.log_tag,
            game_id,
            versions.len()
        );

        Ok(versions)
    }

    async fn download_version(
        &self,
        game_id: String,
        version_id: String,
        target_path: PathBuf,
    ) -> Result<(), CloudError> {
        self.ensure_device_registered().await?;
        let base_url = self.validate_base_url()?;
        let auth = self.get_auth_header()?;

        if let Some(parent) = target_path.parent() {
            fs::create_dir_all(parent).map_err(|e| CloudError::Io(e.to_string()))?;
        }

        let mut resp = self
            .client
            .get(format!("{}/save/download", base_url))
            .header("Authorization", auth)
            .query(&[("game_id", game_id), ("version_id", version_id)])
            .send()
            .await
            .map_err(|e| CloudError::NetworkError(e.to_string()))?;

        if !resp.status().is_success() {
            error!(
                "{} Download failed with status {}",
                self.log_tag,
                resp.status()
            );
            return Err(CloudError::NetworkError(format!(
                "download failed: {}",
                resp.status()
            )));
        }

        let mut file = tokio::fs::File::create(&target_path)
            .await
            .map_err(|e| CloudError::Io(e.to_string()))?;
        while let Some(chunk) = resp
            .chunk()
            .await
            .map_err(|e| CloudError::NetworkError(e.to_string()))?
        {
            file.write_all(&chunk)
                .await
                .map_err(|e| CloudError::Io(e.to_string()))?;
        }

        Ok(())
    }

    fn ensure_device_id(&self) -> Result<String, CloudError> {
        let settings = self
            .settings
            .get_settings()
            .map_err(|e| CloudError::InvalidConfig(format!("settings load failed: {e}")))?;

        if self.mode == CloudMode::Off {
            return Err(CloudError::Disabled);
        }

        if !settings.cloud.enabled && self.mode == CloudMode::Official {
            return Err(CloudError::NotEnabled);
        }

        let (device_id, _, _) = self.ensure_local_device_identity()?;

        if device_id.trim().is_empty() {
            return Err(CloudError::InvalidConfig("device_id missing".into()));
        }

        if settings.cloud.device_id.trim().is_empty() {
            let mut updated = settings.clone();
            updated.cloud.device_id = device_id.clone();
            let _ = self.settings.update_settings(updated);
        }

        Ok(device_id)
    }

    async fn list_devices(&self, token: String) -> Result<Vec<CloudDevice>, CloudError> {
        let base_url = self.validate_base_url()?;
        let resp = self
            .apply_access_headers(
                self.client
                    .get(format!("{}/device/list", base_url))
                    .header("Authorization", format!("Bearer {}", token)),
            )
            .send()
            .await
            .map_err(|e| CloudError::NetworkError(e.to_string()))?;

        if resp.status() == reqwest::StatusCode::UNAUTHORIZED {
            return Err(CloudError::Unauthorized("invalid token".into()));
        }

        if !resp.status().is_success() {
            return Err(CloudError::NetworkError(format!(
                "list devices failed: {}",
                resp.status()
            )));
        }

        let parsed: ListDevicesResponse = resp
            .json()
            .await
            .map_err(|e| CloudError::Serialization(e.to_string()))?;

        Ok(parsed.devices)
    }

    async fn register_device(
        &self,
        token: String,
        device_id: String,
        platform: String,
        device_name: String,
    ) -> Result<(), CloudError> {
        let base_url = self.validate_base_url()?;
        let resp = self
            .apply_access_headers(
                self.client
                    .post(format!("{}/device/register", base_url))
                    .header("Authorization", format!("Bearer {}", token))
                    .json(&serde_json::json!({
                        "device_id": device_id.clone(),
                        "platform": platform.clone(),
                        "device_name": device_name.clone(),
                    })),
            )
            .send()
            .await
            .map_err(|e| CloudError::NetworkError(e.to_string()))?;

        if resp.status() == reqwest::StatusCode::UNAUTHORIZED {
            return Err(CloudError::Unauthorized("invalid token".into()));
        }

        if !resp.status().is_success() {
            return Err(CloudError::NetworkError(format!(
                "register device failed: {}",
                resp.status()
            )));
        }

        let mut app_settings = self
            .settings
            .get_settings()
            .map_err(|e| CloudError::InvalidConfig(format!("settings load failed: {e}")))?;
        app_settings.cloud.device_id = device_id;
        if app_settings.cloud.platform.trim().is_empty() {
            app_settings.cloud.platform = platform;
        }
        if app_settings.cloud.device_name.trim().is_empty() {
            app_settings.cloud.device_name = device_name;
        }
        app_settings.cloud.has_registered_device = true;
        self.settings
            .update_settings(app_settings)
            .map_err(|e| CloudError::InvalidConfig(format!("settings save failed: {e}")))?;

        Ok(())
    }

    async fn remove_device(&self, token: String, device_id: String) -> Result<(), CloudError> {
        let base_url = self.validate_base_url()?;
        let resp = self
            .apply_access_headers(
                self.client
                    .post(format!("{}/device/remove", base_url))
                    .header("Authorization", format!("Bearer {}", token))
                    .json(&serde_json::json!({"device_id": device_id.clone()})),
            )
            .send()
            .await
            .map_err(|e| CloudError::NetworkError(e.to_string()))?;

        if !resp.status().is_success() {
            return Err(CloudError::NetworkError(format!(
                "remove device failed: {}",
                resp.status()
            )));
        }

        let mut app_settings = self
            .settings
            .get_settings()
            .map_err(|e| CloudError::InvalidConfig(format!("settings load failed: {e}")))?;
        if app_settings.cloud.device_id == device_id {
            app_settings.cloud.device_id.clear();
            self.settings
                .update_settings(app_settings)
                .map_err(|e| CloudError::InvalidConfig(format!("settings save failed: {e}")))?;
        }

        Ok(())
    }

    fn get_device_id(&self) -> Result<String, CloudError> {
        let config = self
            .settings
            .get_settings()
            .map_err(|e| CloudError::InvalidConfig(format!("settings load failed: {e}")))?
            .cloud;
        if config.device_id.trim().is_empty() {
            return Err(CloudError::NotFound("device_id not registered".into()));
        }
        Ok(config.device_id)
    }
    
    async fn check_connection(&self) -> Result<bool, CloudError> {
        let base_url = match self.validate_base_url() {
            Ok(url) => url,
            Err(_) => {
                debug!("[CLOUD] check_connection: Invalid base URL");
                return Ok(false); // Invalid config = not connected
            }
        };
        
        debug!("[CLOUD] check_connection: Checking {}/api/games", base_url);
        
        // Simple HEAD request to check if server is reachable
        // ANY HTTP response (even 404) means server is online
        match self.client
            .head(format!("{}/api/games", base_url))
            .timeout(Duration::from_secs(3))
            .send()
            .await
        {
            Ok(resp) => {
                // Any response means server is reachable = online
                // This includes 404, 401, 500, etc.
                debug!("[CLOUD] check_connection: Got response status={}, server is ONLINE", resp.status());
                Ok(true)
            }
            Err(e) => {
                // Only network/timeout errors mean offline
                debug!("[CLOUD] check_connection: Request failed: {}, server is OFFLINE", e);
                Ok(false)
            }
        }
    }

    async fn list_games(&self) -> Result<Vec<String>, CloudError> {
        let base_url = self.validate_base_url()?;
        let auth = self.get_auth_header()?;

        let resp = self
            .apply_access_headers(
                self.client
                    .post(format!("{}/save/games", base_url))
                    .header("Authorization", auth),
            )
            .send()
            .await
            .map_err(|e| CloudError::NetworkError(e.to_string()))?;

        if resp.status() == reqwest::StatusCode::UNAUTHORIZED {
            return Err(CloudError::Unauthorized("invalid token".into()));
        }

        if !resp.status().is_success() {
            let status = resp.status();
            error!(
                "{} list_games failed: status={}",
                self.log_tag, status
            );
            return Err(CloudError::NetworkError(format!(
                "list games failed: {}",
                status
            )));
        }

        #[derive(Deserialize)]
        struct GamesResponse {
            ok: bool,
            games: Vec<String>,
        }

        let parsed: GamesResponse = resp
            .json()
            .await
            .map_err(|e| CloudError::Serialization(e.to_string()))?;

        info!(
            "{} list_games count={}",
            self.log_tag,
            parsed.games.len()
        );

        Ok(parsed.games)
    }
}

// =============================================================================
// Utility helpers
// =============================================================================

pub fn default_device_id() -> String {
    Uuid::new_v4().to_string()
}

pub fn default_device_name(platform: &str) -> String {
    let label = match platform.to_lowercase().as_str() {
        "macos" | "mac" => "Mac Desktop",
        "windows" => "Windows PC",
        "linux" => "Linux Desktop",
        "android" => "Android Device",
        "ios" => "iOS Device",
        other => return format!("{} device", other),
    };
    label.to_string()
}

fn calculate_sha256(path: &PathBuf) -> Result<String, CloudError> {
    let mut file = fs::File::open(path).map_err(|e| CloudError::Io(e.to_string()))?;
    let mut hasher = Sha256::new();
    std::io::copy(&mut file, &mut hasher).map_err(|e| CloudError::Io(e.to_string()))?;
    let result = hasher.finalize();
    Ok(format!("{:x}", result))
}

pub fn log_tag(mode: &CloudMode) -> &'static str {
    match mode {
        CloudMode::Official => "[CLOUD_OFFICIAL]",
        CloudMode::SelfHost => "[CLOUD_SELF_HOST]",
        CloudMode::Off => "[CLOUD_DISABLED]",
    }
}
