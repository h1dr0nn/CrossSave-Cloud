use std::sync::Arc;
use serde::Serialize;
use tracing::info;

use crate::core::history::HistoryManager;
use crate::core::settings::{default_retention_bounds, AppSettings, SettingsError, SettingsManager};

#[derive(Debug, Serialize)]
pub struct StorageInfo {
    pub history_path: String,
    pub size_bytes: u64,
    pub retention_bounds: (usize, usize),
}

fn map_settings_error(err: SettingsError) -> String {
    err.to_string()
}

#[tauri::command]
pub async fn get_app_settings(
    state: tauri::State<'_, Arc<SettingsManager>>,
) -> Result<AppSettings, String> {
    state.get_settings().map_err(map_settings_error)
}

#[tauri::command]
pub async fn update_app_settings(
    state: tauri::State<'_, Arc<SettingsManager>>,
    history: tauri::State<'_, Arc<HistoryManager>>,
    settings: AppSettings,
) -> Result<AppSettings, String> {
    let updated = state.update_settings(settings).map_err(map_settings_error)?;
    if let Err(err) = history.set_policy(updated.retention_limit, updated.auto_delete) {
        return Err(err.to_string());
    }

    Ok(updated)
}

#[tauri::command]
pub async fn get_storage_info(
    history: tauri::State<'_, Arc<HistoryManager>>,
) -> Result<StorageInfo, String> {
    let size_bytes = history.total_size().map_err(|err| err.to_string())?;
    let bounds = default_retention_bounds();
    Ok(StorageInfo {
        history_path: history.base_dir.to_string_lossy().to_string(),
        size_bytes,
        retention_bounds: bounds,
    })
}

#[tauri::command]
pub async fn clear_history_cache(history: tauri::State<'_, Arc<HistoryManager>>) -> Result<(), String> {
    history.clear_all().map_err(|err| err.to_string())?;
    info!("[HISTORY] Cleared history cache by request");
    Ok(())
}
