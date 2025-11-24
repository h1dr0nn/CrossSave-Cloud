use std::fs;
use std::path::PathBuf;
use std::sync::Arc;

use async_trait::async_trait;
use reqwest::{multipart, Client};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use thiserror::Error;
use tokio::io::AsyncWriteExt;
use tracing::{debug, error, info, warn};
use uuid::Uuid;

use crate::core::packager::SaveMetadata;
use crate::core::settings::SettingsManager;

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
    pub name: String,
    pub last_sync: u64,
}

// =============================================================================
// Error Types
// =============================================================================

#[derive(Debug, Error)]
#[allow(dead_code)]
pub enum CloudError {
    #[error("cloud sync is not enabled")]
    NotEnabled,
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
    async fn remove_device(&self, token: String, device_id: String) -> Result<(), CloudError>;
    fn get_device_id(&self) -> Result<String, CloudError>;
}

// =============================================================================
// HTTP Cloud Backend
// =============================================================================

#[derive(Clone)]
pub struct HttpCloudBackend {
    client: Client,
    settings: Arc<SettingsManager>,
}

#[derive(Deserialize)]
struct LoginResponse {
    token: String,
    device_id: Option<String>,
}

#[derive(Deserialize)]
struct RegisterDeviceResponse {
    device_id: String,
}

impl HttpCloudBackend {
    pub fn new(settings: Arc<SettingsManager>) -> Result<Self, CloudError> {
        let config = settings
            .get_settings()
            .map_err(|e| CloudError::InvalidConfig(format!("settings load failed: {e}")))?
            .cloud;
        let timeout = config.timeout_seconds.max(1);
        let client = Client::builder()
            .timeout(std::time::Duration::from_secs(timeout))
            .build()
            .map_err(|e| CloudError::InvalidConfig(format!("client build failed: {e}")))?;

        Ok(Self { client, settings })
    }

    fn validate_base_url(&self) -> Result<String, CloudError> {
        let config = self
            .settings
            .get_settings()
            .map_err(|e| CloudError::InvalidConfig(format!("settings load failed: {e}")))?
            .cloud;
        if config.base_url.trim().is_empty() {
            return Err(CloudError::InvalidConfig("base_url missing".into()));
        }
        Ok(config.base_url.trim_end_matches('/').to_string())
    }

    fn get_auth_header(&self) -> Result<String, CloudError> {
        let config = self
            .settings
            .get_settings()
            .map_err(|e| CloudError::InvalidConfig(format!("settings load failed: {e}")))?
            .cloud;
        if config.api_key.trim().is_empty() {
            return Err(CloudError::Unauthorized("api_key missing".into()));
        }
        Ok(format!("Bearer {}", config.api_key))
    }

    async fn register_device(&self, token: &str) -> Result<String, CloudError> {
        let base_url = self.validate_base_url()?;
        let name = std::env::var("HOSTNAME")
            .ok()
            .filter(|s| !s.is_empty())
            .unwrap_or_else(|| "Current Device".to_string());
        let resp = self
            .client
            .post(format!("{}/device/register", base_url))
            .header("Authorization", token)
            .json(&serde_json::json!({ "name": name }))
            .send()
            .await
            .map_err(|e| CloudError::NetworkError(e.to_string()))?;

        if !resp.status().is_success() {
            return Err(CloudError::NetworkError(format!(
                "device register failed: {}",
                resp.status()
            )));
        }

        let parsed: RegisterDeviceResponse = resp
            .json()
            .await
            .map_err(|e| CloudError::Serialization(e.to_string()))?;

        let mut app_settings = self
            .settings
            .get_settings()
            .map_err(|e| CloudError::InvalidConfig(format!("settings load failed: {e}")))?;
        app_settings.cloud.device_id = parsed.device_id.clone();
        self.settings
            .update_settings(app_settings)
            .map_err(|e| CloudError::InvalidConfig(format!("settings save failed: {e}")))?;

        Ok(parsed.device_id)
    }
}

#[async_trait]
impl CloudBackend for HttpCloudBackend {
    async fn login(&self, email: String, password: String) -> Result<String, CloudError> {
        let base_url = self.validate_base_url()?;
        let resp = self
            .client
            .post(format!("{}/login", base_url))
            .json(&serde_json::json!({ "email": email, "password": password }))
            .send()
            .await
            .map_err(|e| CloudError::NetworkError(e.to_string()))?;

        if !resp.status().is_success() {
            return Err(CloudError::Unauthorized(format!("status {}", resp.status())));
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
            return Err(CloudError::NotFound(format!(
                "archive not found: {}",
                archive_path.display()
            )));
        }

        let base_url = self.validate_base_url()?;
        let auth = self.get_auth_header()?;
        let device_id = self.ensure_device_id()?;

        let archive_size = fs::metadata(&archive_path)
            .map_err(|e| CloudError::Io(e.to_string()))?
            .len();
        let hash = calculate_sha256(&archive_path)?;

        let mut full_metadata = metadata.clone();
        full_metadata.hash = hash.clone();

        let manifest = serde_json::json!({
            "game_id": metadata.game_id,
            "emulator_id": metadata.emulator_id,
            "version_id": metadata.version_id,
            "timestamp": metadata.timestamp,
            "device_id": device_id,
            "file_list": metadata.file_list,
            "hash": hash,
            "total_size_bytes": archive_size,
        });

        let archive_file = fs::read(&archive_path).map_err(|e| CloudError::Io(e.to_string()))?;
        let part = multipart::Part::bytes(archive_file)
            .file_name(format!("{}.zip", full_metadata.version_id));
        let form = multipart::Form::new()
            .text("metadata", manifest.to_string())
            .part("archive", part);

        let resp = self
            .client
            .post(format!("{}/save/upload", base_url))
            .header("Authorization", auth)
            .multipart(form)
            .send()
            .await
            .map_err(|e| CloudError::NetworkError(e.to_string()))?;

        if !resp.status().is_success() {
            return Err(CloudError::NetworkError(format!("upload failed: {}", resp.status())));
        }

        let parsed: CloudVersionSummary = resp
            .json()
            .await
            .map_err(|e| CloudError::Serialization(e.to_string()))?;

        Ok(parsed)
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
        let config = self
            .settings
            .get_settings()
            .map_err(|e| CloudError::InvalidConfig(format!("settings load failed: {e}")))?
            .cloud;
        if !config.enabled {
            return Err(CloudError::NotEnabled);
        }

        if !config.device_id.trim().is_empty() {
            return Ok(config.device_id);
        }

        let token = self.get_auth_header()?;
        futures::executor::block_on(self.register_device(&token))
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

        resp.json()
            .await
            .map_err(|e| CloudError::Serialization(e.to_string()))
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

*** End of File
