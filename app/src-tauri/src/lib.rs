mod api;
mod core;

use api::history_api::{delete_history_item, get_history_item, list_history, rollback_version};
use api::packager_api::{package_save, validate_paths};
use api::profile_api::{delete_profile, get_profile, list_profiles, save_profile};
use api::settings_api::{clear_history_cache, get_app_settings, get_storage_info, update_app_settings};
use api::watcher_api::{start_watcher, stop_watcher};
use core::history::HistoryManager;
use core::profile::{default_profile_dirs, ProfileManager};
use core::settings::{default_settings_path, SettingsManager};
use core::watcher::WatcherManager;
use std::path::PathBuf;

// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let settings_manager = match SettingsManager::new(default_settings_path()) {
        Ok(manager) => manager,
        Err(err) => {
            tracing::error!("[SETTINGS] Failed to load settings: {err}");
            SettingsManager::new(default_settings_path()).expect("failed to initialize settings")
        }
    };

    let current_settings = settings_manager
        .get_settings()
        .unwrap_or_else(|_| core::settings::AppSettings::default());

    let history_base_dir: PathBuf = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("data")
        .join("archives")
        .join("history");
    let history_manager = HistoryManager::init(
        history_base_dir,
        current_settings.retention_limit,
        current_settings.auto_delete,
    )
    .unwrap_or_else(|err| {
        tracing::error!("[HISTORY] Failed to initialize history manager: {err}");
        HistoryManager::with_defaults()
    });

    let (default_profiles, user_profiles) = default_profile_dirs();
    let profile_manager = match ProfileManager::new(default_profiles, user_profiles) {
        Ok(manager) => manager,
        Err(err) => {
            tracing::error!("[PROFILE] Failed to initialize profile manager: {err}");
            let (defaults, users) = default_profile_dirs();
            ProfileManager::new(defaults, users).expect("failed to initialize profiles")
        }
    };

    tauri::Builder::default()
        .manage(WatcherManager::default())
        .manage(history_manager)
        .manage(profile_manager)
        .manage(settings_manager)
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            greet,
            start_watcher,
            stop_watcher,
            list_profiles,
            get_profile,
            save_profile,
            delete_profile,
            package_save,
            validate_paths,
            list_history,
            get_history_item,
            rollback_version,
            delete_history_item,
            get_app_settings,
            update_app_settings,
            get_storage_info,
            clear_history_cache
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
