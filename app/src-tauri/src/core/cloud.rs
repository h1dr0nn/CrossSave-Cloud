use std::fs;
use std::path::PathBuf;
use std::sync::Arc;

use async_trait::async_trait;
use reqwest::{header::CONTENT_TYPE, Client};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use thiserror::Error;
use tokio::io::AsyncWriteExt;
use tracing::{debug, error, info, warn};
use uuid::Uuid;

use crate::core::packager::SaveMetadata;
use crate::core::settings::{CloudMode, SettingsManager};

// =============================================================================
// Configuration
// =============================================================================

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CloudConfig {
    pub enabled: bool,
    pub base_url: String,
    pub api_key: String,
    pub device_id: String,
    pub timeout_seconds: u64,
}

impl Default for CloudConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            base_url: "https://api.crosssave.local".to_string(),
            api_key: String::new(),
            device_id: String::new(),
            timeout_seconds: 30,
        }
    }
}

// =============================================================================
// Cloud Version Summary
// =============================================================================

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CloudVersionSummary {
    pub game_id: String,
    pub emulator_id: String,
    pub version_id: String,
    pub timestamp: u64,
    pub size_bytes: u64,
    pub hash: String,
    pub device_id: String,
    pub file_list: Vec<String>,
    pub total_size_bytes: u64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CloudDevice {
    pub device_id: String,
    pub platform: String,
    pub last_seen: u64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct UploadRequest {
    pub game_id: String,
    pub version_id: String,
    pub size_bytes: u64,
    pub sha256: String,
    pub file_list: Vec<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct UploadUrlResponse {
    pub upload_url: String,
    pub r2_key: String,
    pub version_id: String,
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
    ) -> Result<(), CloudError>;
    async fn remove_device(&self, token: String, device_id: String) -> Result<(), CloudError>;
    fn get_device_id(&self) -> Result<String, CloudError>;
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
    ) -> Result<(), CloudError> {
        Err(CloudError::Disabled)
    }

    async fn remove_device(&self, _token: String, _device_id: String) -> Result<(), CloudError> {
        Err(CloudError::Disabled)
    }

    fn get_device_id(&self) -> Result<String, CloudError> {
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
}

pub type SelfHostHttpBackend = HttpCloudBackend;

#[derive(Deserialize)]
struct LoginResponse {
    token: String,
    device_id: Option<String>,
}

#[derive(Deserialize)]
struct ListDevicesResponse {
    devices: Vec<CloudDevice>,
}

impl HttpCloudBackend {
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

        Ok(Self {
            client,
            settings,
            mode,
            log_tag,
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

    fn ensure_local_device_id(&self) -> Result<String, CloudError> {
        let mut app_settings = self
            .settings
            .get_settings()
            .map_err(|e| CloudError::InvalidConfig(format!("settings load failed: {e}")))?;

        if app_settings.cloud.device_id.trim().is_empty() {
            app_settings.cloud.device_id = default_device_id();
            self.settings
                .update_settings(app_settings.clone())
                .map_err(|e| CloudError::InvalidConfig(format!("settings save failed: {e}")))?;
        }

        Ok(app_settings.cloud.device_id)
    }

    async fn ensure_device_registered(&self) -> Result<String, CloudError> {
        let device_id = self.ensure_local_device_id()?;
        let base_url = self.validate_base_url()?;
        let auth = self.get_auth_header()?;
        let platform = std::env::consts::OS.to_string();

        let resp = self
            .client
            .post(format!("{}/device/register", base_url))
            .header("Authorization", auth)
            .json(&serde_json::json!({ "device_id": device_id, "platform": platform }))
            .send()
            .await
            .map_err(|e| CloudError::NetworkError(e.to_string()))?;

        if resp.status() == reqwest::StatusCode::UNAUTHORIZED {
            return Err(CloudError::Unauthorized("invalid token".into()));
        }

        if !resp.status().is_success() {
            error!("{} device register failed: {}", self.log_tag, resp.status());
            return Err(CloudError::NetworkError(format!(
                "device register failed: {}",
                resp.status()
            )));
        }

        Ok(device_id)
    }
}

#[async_trait]
impl CloudBackend for HttpCloudBackend {
    async fn login(&self, email: String, password: String) -> Result<String, CloudError> {
        info!("{} Attempting login", self.log_tag);
        let base_url = self.validate_base_url()?;
        let resp = self
            .client
            .post(format!("{}/login", base_url))
            .json(&serde_json::json!({ "email": email, "password": password }))
            .send()
            .await
            .map_err(|e| CloudError::NetworkError(e.to_string()))?;

        if !resp.status().is_success() {
            warn!(
                "{} Login failed with status {}",
                self.log_tag,
                resp.status()
            );
            return Err(CloudError::Unauthorized(format!(
                "status {}",
                resp.status(),
            )));
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

        let upload_request = UploadRequest {
            game_id: metadata.game_id.clone(),
            version_id: metadata.version_id.clone(),
            size_bytes: archive_size,
            sha256: hash.clone(),
            file_list: metadata.file_list.clone(),
        };

        let signed = self.request_upload_url(upload_request.clone()).await?;

        let archive_file = fs::read(&archive_path).map_err(|e| CloudError::Io(e.to_string()))?;
        let resp = self
            .client
            .put(&signed.upload_url)
            .header(CONTENT_TYPE, "application/zip")
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
            game_id: upload_request.game_id,
            emulator_id: metadata.emulator_id,
            version_id: upload_request.version_id,
            timestamp: metadata.timestamp,
            size_bytes: upload_request.size_bytes,
            hash,
            device_id,
            file_list: upload_request.file_list,
            total_size_bytes: archive_size,
        })
    }

    async fn request_upload_url(
        &self,
        payload: UploadRequest,
    ) -> Result<UploadUrlResponse, CloudError> {
        let base_url = self.validate_base_url()?;
        let auth = self.get_auth_header()?;

        let resp = self
            .client
            .post(format!("{}/save/upload-url", base_url))
            .header("Authorization", auth)
            .json(&payload)
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
        let base_url = self.validate_base_url()?;
        let auth = self.get_auth_header()?;

        let resp = self
            .client
            .post(format!("{}/save/notify-upload", base_url))
            .header("Authorization", auth)
            .json(&payload)
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

    async fn list_versions(
        &self,
        game_id: String,
        limit: Option<usize>,
    ) -> Result<Vec<CloudVersionSummary>, CloudError> {
        let base_url = self.validate_base_url()?;
        let auth = self.get_auth_header()?;
        let mut request = self
            .client
            .get(format!("{}/save/list", base_url))
            .header("Authorization", auth)
            .query(&[("game_id", game_id.clone())]);
        if let Some(limit) = limit {
            request = request.query(&[("limit", limit.to_string())]);
        }

        let resp = request
            .send()
            .await
            .map_err(|e| CloudError::NetworkError(e.to_string()))?;

        if resp.status() == reqwest::StatusCode::NOT_FOUND {
            return Ok(Vec::new());
        }

        if !resp.status().is_success() {
            return Err(CloudError::NetworkError(format!(
                "list failed: {}",
                resp.status()
            )));
        }

        resp.json()
            .await
            .map_err(|e| CloudError::Serialization(e.to_string()))
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

        if !settings.cloud.device_id.trim().is_empty() {
            return Ok(settings.cloud.device_id);
        }

        futures::executor::block_on(self.ensure_device_registered())
    }

    async fn list_devices(&self, token: String) -> Result<Vec<CloudDevice>, CloudError> {
        let base_url = self.validate_base_url()?;
        let resp = self
            .client
            .get(format!("{}/device/list", base_url))
            .header("Authorization", format!("Bearer {}", token))
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
    ) -> Result<(), CloudError> {
        let base_url = self.validate_base_url()?;
        let resp = self
            .client
            .post(format!("{}/device/register", base_url))
            .header("Authorization", format!("Bearer {}", token))
            .json(
                &serde_json::json!({"device_id": device_id.clone(), "platform": platform.clone()}),
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
        self.settings
            .update_settings(app_settings)
            .map_err(|e| CloudError::InvalidConfig(format!("settings save failed: {e}")))?;

        Ok(())
    }

    async fn remove_device(&self, token: String, device_id: String) -> Result<(), CloudError> {
        let base_url = self.validate_base_url()?;
        let resp = self
            .client
            .post(format!("{}/device/remove", base_url))
            .header("Authorization", format!("Bearer {}", token))
            .json(&serde_json::json!({"device_id": device_id.clone()}))
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
}

// =============================================================================
// Utility helpers
// =============================================================================

pub fn default_device_id() -> String {
    Uuid::new_v4().to_string()
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
