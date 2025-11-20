use std::path::PathBuf;

use tracing::{error, info, warn};

use crate::core::watcher::{SharedWatcherManager, WatcherError};

#[tauri::command]
pub async fn start_watcher(
    app: tauri::AppHandle,
    state: SharedWatcherManager<'_>,
    paths: Vec<String>,
) -> Result<(), String> {
    let resolved_paths: Vec<PathBuf> = paths.into_iter().map(PathBuf::from).collect();
    match state.start(app, resolved_paths) {
        Ok(_) => {
            info!("[WATCHER] Watcher started from API");
            Ok(())
        }
        Err(err) => {
            error!("[WATCHER] Failed to start watcher: {err}");
            Err(err.to_string())
        }
    }
}

#[tauri::command]
pub async fn stop_watcher(state: SharedWatcherManager<'_>) -> Result<(), String> {
    match state.stop().await {
        Ok(_) => {
            info!("[WATCHER] Watcher stopped from API");
            Ok(())
        }
        Err(WatcherError::NotRunning) => {
            warn!("[WATCHER] Stop requested but watcher not running");
            Ok(())
        }
        Err(err) => {
            error!("[WATCHER] Failed to stop watcher: {err}");
            Err(err.to_string())
        }
    }
}
