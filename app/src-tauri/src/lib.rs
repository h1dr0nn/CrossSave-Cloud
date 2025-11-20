mod api;
mod core;

use api::history_api::{delete_history_item, get_history_item, list_history, rollback_version};
use api::packager_api::{package_save, validate_paths};
use api::profile_api::{get_profile, list_profiles};
use api::watcher_api::{start_watcher, stop_watcher};
use core::history::HistoryManager;
use core::profile;
use core::watcher::WatcherManager;

// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    if let Err(err) = profile::load_profiles() {
        tracing::error!("[PROFILE] Failed to load emulator profiles: {err}");
    }

    tauri::Builder::default()
        .manage(WatcherManager::default())
        .manage(HistoryManager::default())
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            greet,
            start_watcher,
            stop_watcher,
            list_profiles,
            get_profile,
            package_save,
            validate_paths,
            list_history,
            get_history_item,
            rollback_version,
            delete_history_item
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
