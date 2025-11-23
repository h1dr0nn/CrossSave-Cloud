use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::Mutex;

use serde::Serialize;
use tauri::{Manager, State};

use crate::core::cloud::{CloudBackend, CloudError, CloudVersionSummary};
use crate::core::history::HistoryManager;
use crate::core::settings::{CloudSettings, SettingsManager};

#[derive(Clone, Debug, Serialize)]
pub struct CloudStatus {
    pub enabled: bool,
    pub device_id: String,
    pub connected: bool,
}

#[tauri::command]
pub async fn upload_cloud_save(
    game_id: String,
    _emulator_id: String,
    local_version_id: String,
    cloud: State<'_, Arc<Mutex<Box<dyn CloudBackend + Send>>>>,
    history: State<'_, HistoryManager>,
) -> Result<CloudVersionSummary, String> {
    let history_entry = history
        .get_history_item(game_id, local_version_id)
        .map_err(|e| format!("Failed to find local version: {e}"))?;

    let archive_path_buf = PathBuf::from(&history_entry.archive_path);
    let metadata = history_entry.metadata;
    
    let backend = cloud.lock().await;
    backend.upload_archive(metadata, archive_path_buf).await.map_err(cloud_error_to_string)
}

#[tauri::command]
pub async fn list_cloud_versions(
    game_id: String,
    limit: Option<u32>,
    cloud: State<'_, Arc<Mutex<Box<dyn CloudBackend + Send>>>>,
) -> Result<Vec<CloudVersionSummary>, String> {
    let backend = cloud.lock().await;
    backend.list_versions(game_id, limit.map(|l| l as usize)).await.map_err(cloud_error_to_string)
}

#[tauri::command]
pub async fn download_cloud_version(
    game_id: String,
    version_id: String,
    cloud: State<'_, Arc<Mutex<Box<dyn CloudBackend + Send>>>>,
    app: tauri::AppHandle,
) -> Result<String, String> {
    let app_data_dir = app.path().app_data_dir().map_err(|e| format!("Failed to get app data dir: {e}"))?;
    let downloads_dir = app_data_dir.join("data").join("cloud_downloads");
    let target_path = downloads_dir.join(format!("{}_{}.zip", game_id, version_id));
    
    let backend = cloud.lock().await;
    backend.download_version(game_id, version_id, target_path.clone()).await.map_err(cloud_error_to_string)?;
    
    Ok(target_path.to_string_lossy().to_string())
}

#[tauri::command]
pub async fn get_cloud_config(
    settings: State<'_, SettingsManager>,
) -> Result<CloudSettings, String> {
    settings.get_settings()
        .map(|s| s.cloud)
        .map_err(|e| format!("Failed to load settings: {e}"))
}

#[tauri::command]
pub async fn update_cloud_config(
    new_config: CloudSettings,
    settings: State<'_, SettingsManager>,
) -> Result<CloudSettings, String> {
    let mut app_settings = settings.get_settings().map_err(|e| format!("Failed to load settings: {e}"))?;
    app_settings.cloud = new_config;
    settings.update_settings(app_settings).map(|s| s.cloud).map_err(|e| format!("Failed to save settings: {e}"))
}

#[tauri::command]
pub async fn get_cloud_status(
    cloud: State<'_, Arc<Mutex<Box<dyn CloudBackend + Send>>>>,
    settings: State<'_, SettingsManager>,
) -> Result<CloudStatus, String> {
    let app_settings = settings.get_settings().map_err(|e| format!("Failed to load settings: {e}"))?;
    let backend = cloud.lock().await;
    let device_id = backend.ensure_device_id().map_err(cloud_error_to_string)?;

    Ok(CloudStatus {
        enabled: app_settings.cloud.enabled,
        device_id,
        connected: true,
    })
}

fn cloud_error_to_string(error: CloudError) -> String {
    match error {
        CloudError::NotEnabled => "Cloud sync is not enabled".to_string(),
        CloudError::NetworkError(msg) => format!("Network error: {}", msg),
        CloudError::StorageError(msg) => format!("Storage error: {}", msg),
        CloudError::NotFound(msg) => format!("Not found: {}", msg),
        CloudError::InvalidConfig(msg) => format!("Invalid configuration: {}", msg),
        CloudError::Io(msg) => format!("IO error: {}", msg),
        CloudError::Serialization(msg) => format!("Serialization error: {}", msg),
    }
}
