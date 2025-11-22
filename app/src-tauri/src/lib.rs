mod api;
mod core;

use api::explorer_api::{check_path_status, scan_save_files};
use api::history_api::{delete_history_item, get_history_item, list_history, rollback_version};
use api::packager_api::{package_save, validate_paths};
use api::profile_api::{delete_profile, get_profile, list_profiles, save_profile};
use api::settings_api::{
    clear_history_cache, get_app_settings, get_storage_info, update_app_settings,
};
use api::watcher_api::{start_watcher, stop_watcher};
use core::history::HistoryManager;
use core::profile::ProfileManager;
use core::settings::SettingsManager;
use core::watcher::WatcherManager;
use std::path::PathBuf;
use std::sync::{Arc, RwLock};
use tauri::Manager;

// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tracing::info!("[STARTUP] Rust backend starting...");
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            tracing::info!("[STARTUP] Tauri setup hook running...");
            // Get app data directory (works on all platforms including Android)
            let app_data_dir = app
                .path()
                .app_data_dir()
                .expect("failed to get app data directory");

            // Settings path
            let settings_path = app_data_dir.join("config").join("settings.json");
            let settings_manager = match SettingsManager::new(settings_path) {
                Ok(manager) => manager,
                Err(err) => {
                    tracing::error!("[SETTINGS] Failed to load settings: {err}");
                    // Fallback to default
                    SettingsManager::new(app_data_dir.join("config").join("settings.json"))
                        .expect("failed to initialize settings")
                }
            };

            let current_settings = settings_manager
                .get_settings()
                .unwrap_or_else(|_| core::settings::AppSettings::default());

            // History directory
            let history_base_dir = app_data_dir.join("archives").join("history");
            let history_manager = HistoryManager::init(
                history_base_dir,
                current_settings.retention_limit,
                current_settings.auto_delete,
            )
            .unwrap_or_else(|err| {
                tracing::error!("[HISTORY] Failed to initialize history manager: {err}");
                HistoryManager::with_defaults()
            });

            // Profile directories
            let (default_profiles, user_profiles) = default_profile_dirs_for_app(app);
            let profile_manager = match ProfileManager::new(default_profiles, user_profiles) {
                Ok(manager) => manager,
                Err(err) => {
                    tracing::error!("[PROFILE] Failed to initialize profile manager: {err}");
                    let (defaults, users) = default_profile_dirs_for_app(app);
                    ProfileManager::new(defaults, users).expect("failed to initialize profiles")
                }
            };

            // Register state
            app.manage(WatcherManager::default());
            app.manage(history_manager);
            app.manage(Arc::new(RwLock::new(profile_manager)));
            app.manage(settings_manager);

            Ok(())
        })
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
            clear_history_cache,
            scan_save_files,
            check_path_status
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

fn default_profile_dirs_for_app(app: &tauri::App) -> (PathBuf, PathBuf) {
    let app_data_dir = app
        .path()
        .app_data_dir()
        .expect("failed to get app data directory");

    // On Android, we can't bundle default profiles in resources
    // So we'll use app data dir for both default and user profiles
    #[cfg(target_os = "android")]
    {
        let profiles_dir = app_data_dir.join("profiles");

        // Create default RetroArch profile on first run
        if let Err(e) = std::fs::create_dir_all(&profiles_dir) {
            tracing::error!("[PROFILE] Failed to create profiles dir: {e}");
        } else {
            let retroarch_profile = profiles_dir.join("retroarch.json");
            if !retroarch_profile.exists() {
                let default_profile = r#"{
  "emulator_id": "retroarch",
  "name": "RetroArch",
  "default_save_paths": [
    "/storage/emulated/0/RetroArch/saves",
    "/sdcard/RetroArch/saves",
    "/storage/sdcard1/RetroArch/saves",
    "/mnt/media_rw/sdcard1/RetroArch/saves"
  ],
  "file_patterns": ["*.srm", "*.sav", "*.state"]
}"#;
                let _ = std::fs::write(&retroarch_profile, default_profile);
            }

            let drastic_profile = profiles_dir.join("drastic.json");
            if !drastic_profile.exists() {
                let default_profile = r#"{
  "emulator_id": "drastic",
  "name": "DraStic (DS)",
  "default_save_paths": [
    "/storage/emulated/0/DraStic/backup",
    "/sdcard/DraStic/backup",
    "/storage/sdcard1/DraStic/backup",
    "/mnt/media_rw/sdcard1/DraStic/backup"
  ],
  "file_patterns": ["*.dsv"]
}"#;
                let _ = std::fs::write(&drastic_profile, default_profile);
            }

            let ppsspp_profile = profiles_dir.join("ppsspp.json");
            if !ppsspp_profile.exists() {
                let default_profile = r#"{
  "emulator_id": "ppsspp",
  "name": "PPSSPP (PSP)",
  "default_save_paths": [
    "/storage/emulated/0/PSP/SAVEDATA",
    "/sdcard/PSP/SAVEDATA",
    "/storage/sdcard1/PSP/SAVEDATA",
    "/mnt/media_rw/sdcard1/PSP/SAVEDATA"
  ],
  "file_patterns": ["*.ini", "PARAM.SFO"]
}"#;
                let _ = std::fs::write(&ppsspp_profile, default_profile);
            }

            let myboy_profile = profiles_dir.join("myboy.json");
            if !myboy_profile.exists() {
                let default_profile = r#"{
  "emulator_id": "myboy",
  "name": "MyBoy! (GBA)",
  "default_save_paths": [
    "/storage/emulated/0/MyBoy/save",
    "/sdcard/MyBoy/save",
    "/storage/sdcard1/MyBoy/save",
    "/mnt/media_rw/sdcard1/MyBoy/save"
  ],
  "file_patterns": ["*.sav", "*.st*"]
}"#;
                let _ = std::fs::write(&myboy_profile, default_profile);
            }

            let duckstation_profile = profiles_dir.join("duckstation.json");
            if !duckstation_profile.exists() {
                let default_profile = r#"{
  "emulator_id": "duckstation",
  "name": "DuckStation (PS1)",
  "default_save_paths": [
    "/storage/emulated/0/duckstation/memcards",
    "/sdcard/duckstation/memcards",
    "/storage/sdcard1/duckstation/memcards",
    "/mnt/media_rw/sdcard1/duckstation/memcards"
  ],
  "file_patterns": ["*.mcd", "*.mcr"]
}"#;
                let _ = std::fs::write(&duckstation_profile, default_profile);
            }

            let aethersx2_profile = profiles_dir.join("aethersx2.json");
            if !aethersx2_profile.exists() {
                let default_profile = r#"{
  "emulator_id": "aethersx2",
  "name": "AetherSX2 (PS2)",
  "default_save_paths": [
    "/storage/emulated/0/AetherSX2/memcards",
    "/sdcard/AetherSX2/memcards",
    "/storage/emulated/0/Android/data/xyz.aethersx2.android/files/memcards",
    "/storage/sdcard1/AetherSX2/memcards",
    "/mnt/media_rw/sdcard1/AetherSX2/memcards"
  ],
  "file_patterns": ["*.ps2"]
}"#;
                let _ = std::fs::write(&aethersx2_profile, default_profile);
            }

            let dolphin_profile = profiles_dir.join("dolphin.json");
            if !dolphin_profile.exists() {
                let default_profile = r#"{
  "emulator_id": "dolphin",
  "name": "Dolphin (GC/Wii)",
  "default_save_paths": [
    "/storage/emulated/0/dolphin-emu/GC",
    "/sdcard/dolphin-emu/GC",
    "/storage/sdcard1/dolphin-emu/GC",
    "/mnt/media_rw/sdcard1/dolphin-emu/GC",
    "/storage/emulated/0/dolphin-emu/Wii/title/00010000",
    "/sdcard/dolphin-emu/Wii/title/00010000",
    "/storage/sdcard1/dolphin-emu/Wii/title/00010000",
    "/mnt/media_rw/sdcard1/dolphin-emu/Wii/title/00010000"
  ],
  "file_patterns": ["*.gci", "*.sav", "*.bin"]
}"#;
                let _ = std::fs::write(&dolphin_profile, default_profile);
            }
        }

        (profiles_dir.clone(), profiles_dir)
    }

    #[cfg(not(target_os = "android"))]
    {
        let base = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        let default_dir = base.join("resources").join("profiles");
        let user_dir = app_data_dir.join("profiles");
        (default_dir, user_dir)
    }
}
