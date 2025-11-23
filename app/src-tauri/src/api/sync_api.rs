
use tauri::State;
use crate::core::sync::{SyncManager, SyncStatus};

#[tauri::command]
pub async fn get_sync_status(
    sync: State<'_, SyncManager>,
) -> Result<SyncStatus, String> {
    Ok(sync.queue.get_status().await)
}

#[tauri::command]
pub async fn force_sync_now(
    sync: State<'_, SyncManager>,
) -> Result<(), String> {
    sync.trigger_sync();
    Ok(())
}

#[tauri::command]
pub async fn clear_sync_queue(
    sync: State<'_, SyncManager>,
) -> Result<(), String> {
    sync.queue.clear().await;
    Ok(())
}
