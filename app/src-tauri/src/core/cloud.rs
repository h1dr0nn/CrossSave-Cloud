use std::fs;
use std::path::PathBuf;

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use thiserror::Error;
use tracing::{debug, error, info, warn};
use uuid::Uuid;

use crate::core::packager::SaveMetadata;

// ============================================================================
// Configuration
// ============================================================================

#[derive(Clone, Debug, Serialize, Deserialize)]
#[allow(dead_code)]
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

// ============================================================================
// Cloud Version Summary
// ============================================================================

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CloudVersionSummary {
    pub game_id: String,
    pub emulator_id: String,
    pub version_id: String,
    pub timestamp: u64,
    pub size_bytes: u64,
    pub hash: String,
    pub device_id: String,
}

// ============================================================================
// Error Types
// ============================================================================

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
}

// ============================================================================
// Cloud Backend Trait
// ============================================================================

#[async_trait]
pub trait CloudBackend: Send + Sync {
    /// Upload a save archive to cloud storage
    async fn upload_archive(
        &self,
        metadata: SaveMetadata,
        archive_path: PathBuf,
    ) -> Result<CloudVersionSummary, CloudError>;

    /// List all versions for a specific game
    async fn list_versions(
        &self,
        game_id: String,
        limit: Option<usize>,
    ) -> Result<Vec<CloudVersionSummary>, CloudError>;

    /// Download a specific version from cloud storage
    async fn download_version(
        &self,
        game_id: String,
        version_id: String,
        target_path: PathBuf,
    ) -> Result<(), CloudError>;

    /// Ensure device ID exists and return it
    fn ensure_device_id(&self) -> Result<String, CloudError>;
}

// ============================================================================
// Mock Cloud Backend (Filesystem-based)
// ============================================================================

#[derive(Clone, Debug, Serialize, Deserialize)]
struct GameManifest {
    game_id: String,
    versions: Vec<CloudVersionSummary>,
}

pub struct MockCloudBackend {
    storage_dir: PathBuf,
    #[allow(dead_code)]
    downloads_dir: PathBuf,
    device_id: String,
}

impl MockCloudBackend {
    pub fn new(storage_dir: PathBuf, downloads_dir: PathBuf) -> Self {
        // Generate device ID if not exists
        let device_id = Self::load_or_generate_device_id(&storage_dir);

        info!(
            "[CLOUD] MockCloudBackend initialized (storage={}, downloads={}, device_id={})",
            storage_dir.display(),
            downloads_dir.display(),
            device_id
        );

        Self {
            storage_dir,
            downloads_dir,
            device_id,
        }
    }

    fn load_or_generate_device_id(storage_dir: &PathBuf) -> String {
        let device_id_file = storage_dir.join("device_id.txt");

        if device_id_file.exists() {
            if let Ok(id) = fs::read_to_string(&device_id_file) {
                let trimmed = id.trim();
                if !trimmed.is_empty() {
                    debug!("[CLOUD] Loaded existing device_id from {}", device_id_file.display());
                    return trimmed.to_string();
                }
            }
        }

        // Generate new device ID
        let new_id = Uuid::new_v4().to_string();
        
        // Try to persist it
        if let Err(e) = fs::create_dir_all(storage_dir) {
            warn!("[CLOUD] Failed to create storage dir for device_id: {e}");
        } else if let Err(e) = fs::write(&device_id_file, &new_id) {
            warn!("[CLOUD] Failed to persist device_id: {e}");
        } else {
            info!("[CLOUD] Generated and saved new device_id: {}", new_id);
        }

        new_id
    }

    fn game_dir(&self, game_id: &str) -> PathBuf {
        self.storage_dir.join(game_id)
    }

    fn manifest_path(&self, game_id: &str) -> PathBuf {
        self.game_dir(game_id).join("manifest.json")
    }

    fn archive_path(&self, game_id: &str, version_id: &str) -> PathBuf {
        self.game_dir(game_id).join(format!("{}.zip", version_id))
    }

    fn load_manifest(&self, game_id: &str) -> Result<GameManifest, CloudError> {
        let manifest_path = self.manifest_path(game_id);

        if !manifest_path.exists() {
            debug!("[CLOUD] No manifest exists for game_id={}", game_id);
            return Ok(GameManifest {
                game_id: game_id.to_string(),
                versions: Vec::new(),
            });
        }

        let content = fs::read_to_string(&manifest_path)
            .map_err(|e| CloudError::Io(format!("Failed to read manifest: {e}")))?;

        let manifest: GameManifest = serde_json::from_str(&content)
            .map_err(|e| CloudError::Serialization(format!("Failed to parse manifest: {e}")))?;

        Ok(manifest)
    }

    fn save_manifest(&self, manifest: &GameManifest) -> Result<(), CloudError> {
        let manifest_path = self.manifest_path(&manifest.game_id);

        // Ensure game directory exists
        if let Some(parent) = manifest_path.parent() {
            fs::create_dir_all(parent)
                .map_err(|e| CloudError::Io(format!("Failed to create game dir: {e}")))?;
        }

        let json = serde_json::to_string_pretty(manifest)
            .map_err(|e| CloudError::Serialization(format!("Failed to serialize manifest: {e}")))?;

        fs::write(&manifest_path, json)
            .map_err(|e| CloudError::Io(format!("Failed to write manifest: {e}")))?;

        debug!(
            "[CLOUD] Saved manifest for game_id={} ({} versions)",
            manifest.game_id,
            manifest.versions.len()
        );

        Ok(())
    }
}

#[async_trait]
impl CloudBackend for MockCloudBackend {
    async fn upload_archive(
        &self,
        metadata: SaveMetadata,
        archive_path: PathBuf,
    ) -> Result<CloudVersionSummary, CloudError> {
        info!(
            "[CLOUD] Uploading archive for game_id={}, version_id={}",
            metadata.game_id, metadata.version_id
        );

        if !archive_path.exists() {
            return Err(CloudError::NotFound(format!(
                "Archive not found at {}",
                archive_path.display()
            )));
        }

        // Get file size
        let file_metadata = fs::metadata(&archive_path)
            .map_err(|e| CloudError::Io(format!("Failed to read archive metadata: {e}")))?;

        let size_bytes = file_metadata.len();

        // Create cloud version summary
        let summary = CloudVersionSummary {
            game_id: metadata.game_id.clone(),
            emulator_id: metadata.emulator_id.clone(),
            version_id: metadata.version_id.clone(),
            timestamp: metadata.timestamp,
            size_bytes,
            hash: metadata.hash.clone(),
            device_id: self.device_id.clone(),
        };

        // Copy archive to mock cloud storage
        let target_path = self.archive_path(&metadata.game_id, &metadata.version_id);
        if let Some(parent) = target_path.parent() {
            fs::create_dir_all(parent)
                .map_err(|e| CloudError::Io(format!("Failed to create target dir: {e}")))?;
        }

        fs::copy(&archive_path, &target_path)
            .map_err(|e| CloudError::Io(format!("Failed to copy archive: {e}")))?;

        debug!(
            "[CLOUD] Copied archive to {}",
            target_path.display()
        );

        // Update manifest
        let mut manifest = self.load_manifest(&metadata.game_id)?;
        
        // Remove existing version with same ID (if any)
        manifest.versions.retain(|v| v.version_id != summary.version_id);
        
        // Add new version
        manifest.versions.push(summary.clone());
        
        // Sort by timestamp (newest first)
        manifest.versions.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));
        
        self.save_manifest(&manifest)?;

        info!(
            "[CLOUD] Successfully uploaded version_id={} ({} bytes)",
            summary.version_id, summary.size_bytes
        );

        Ok(summary)
    }

    async fn list_versions(
        &self,
        game_id: String,
        limit: Option<usize>,
    ) -> Result<Vec<CloudVersionSummary>, CloudError> {
        debug!("[CLOUD] Listing versions for game_id={}", game_id);

        let manifest = self.load_manifest(&game_id)?;
        
        let mut versions = manifest.versions;
        
        // Apply limit if specified
        if let Some(limit) = limit {
            versions.truncate(limit);
        }

        debug!(
            "[CLOUD] Found {} versions for game_id={}",
            versions.len(),
            game_id
        );

        Ok(versions)
    }

    async fn download_version(
        &self,
        game_id: String,
        version_id: String,
        target_path: PathBuf,
    ) -> Result<(), CloudError> {
        info!(
            "[CLOUD] Downloading version_id={} for game_id={}",
            version_id, game_id
        );

        let source_path = self.archive_path(&game_id, &version_id);

        if !source_path.exists() {
            return Err(CloudError::NotFound(format!(
                "Version {}/{} not found in cloud storage",
                game_id, version_id
            )));
        }

        // Ensure target directory exists
        if let Some(parent) = target_path.parent() {
            fs::create_dir_all(parent)
                .map_err(|e| CloudError::Io(format!("Failed to create target dir: {e}")))?;
        }

        fs::copy(&source_path, &target_path)
            .map_err(|e| CloudError::Io(format!("Failed to download archive: {e}")))?;

        info!(
            "[CLOUD] Downloaded to {}",
            target_path.display()
        );

        Ok(())
    }

    fn ensure_device_id(&self) -> Result<String, CloudError> {
        Ok(self.device_id.clone())
    }
}
