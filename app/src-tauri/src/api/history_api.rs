use std::sync::Arc;
use tracing::{error, warn};

use crate::core::history::{HistoryEntry, HistoryManager};
use crate::core::packager::PackagedSave;

fn sanitize_input(value: String, field: &str) -> Result<String, String> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        Err(format!("{field} cannot be empty"))
    } else {
        Ok(trimmed.to_string())
    }
}

#[tauri::command]
pub async fn list_history(
    state: tauri::State<'_, Arc<HistoryManager>>,
    game_id: String,
) -> Result<Vec<HistoryEntry>, String> {
    let sanitized_game_id = sanitize_input(game_id, "game_id")?;

    state.list_history(sanitized_game_id).map_err(|err| {
        error!("[HISTORY] Failed to list history: {err}");
        err.to_string()
    })
}

#[tauri::command]
pub async fn get_history_item(
    state: tauri::State<'_, Arc<HistoryManager>>,
    game_id: String,
    version_id: String,
) -> Result<HistoryEntry, String> {
    let sanitized_game_id = sanitize_input(game_id, "game_id")?;
    let sanitized_version_id = sanitize_input(version_id, "version_id")?;

    state
        .get_history_item(sanitized_game_id, sanitized_version_id)
        .map_err(|err| {
            error!("[HISTORY] Failed to fetch history item: {err}");
            err.to_string()
        })
}

#[tauri::command]
pub async fn rollback_version(
    state: tauri::State<'_, Arc<HistoryManager>>,
    game_id: String,
    version_id: String,
) -> Result<PackagedSave, String> {
    let sanitized_game_id = sanitize_input(game_id, "game_id")?;
    let sanitized_version_id = sanitize_input(version_id, "version_id")?;

    state
        .rollback_version(sanitized_game_id, sanitized_version_id)
        .map_err(|err| {
            error!("[HISTORY] Rollback failed: {err}");
            err.to_string()
        })
}

#[tauri::command]
pub async fn delete_history_item(
    state: tauri::State<'_, Arc<HistoryManager>>,
    game_id: String,
    version_id: String,
) -> Result<(), String> {
    let sanitized_game_id = sanitize_input(game_id, "game_id")?;
    let sanitized_version_id = sanitize_input(version_id, "version_id")?;

    state
        .delete_history_item(sanitized_game_id, sanitized_version_id)
        .map_err(|err| {
            warn!("[HISTORY] Failed to delete history item: {err}");
            err.to_string()
        })
}
