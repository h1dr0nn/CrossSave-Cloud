mod api;
mod core;

use api::cloud_api::{
    download_cloud_save, download_cloud_version, get_cloud_config, get_cloud_status,
    get_conflict_details, get_upload_url, list_cloud_devices, list_cloud_versions, login_cloud,
    logout_cloud, notify_upload, reconnect_cloud, register_cloud_device, remove_cloud_device,
    resolve_conflict_download, resolve_conflict_upload, signup_cloud, update_cloud_config,
    update_cloud_mode, upload_cloud_save, validate_official_cloud_settings,
    validate_self_host_settings,
};
use api::explorer_api::{check_path_status, open_folder, scan_save_files};
use api::history_api::{delete_history_item, get_history_item, list_history, rollback_version};
use api::packager_api::{package_game, package_save, validate_paths};
use api::profile_api::{delete_profile, get_profile, list_profiles, save_profile};
use api::settings_api::{
    clear_history_cache, get_app_settings, get_storage_info, update_app_settings,
};
use api::sync_api::{clear_sync_queue, force_sync_now, get_sync_status};
use api::watcher_api::{start_watcher, stop_watcher};
use core::cloud::{
    default_device_id, log_tag, CloudBackend, CloudError, DisabledCloudBackend, HttpCloudBackend,
};
use core::history::HistoryManager;
use core::profile::ProfileManager;
use core::settings::{AppSettings, CloudMode, SettingsManager};
use core::sync::SyncManager;
use core::watcher::WatcherManager;
use std::path::PathBuf;
use std::sync::{Arc, RwLock};
use tracing::info;
use tauri::Emitter;
use tauri::Manager;
use tokio::sync::Mutex;

type CloudBackendState = Arc<Mutex<Box<dyn CloudBackend + Send>>>;

// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[tauri::command]
async fn select_directory(app_handle: tauri::AppHandle) -> Result<Option<String>, String> {
    use tauri_plugin_dialog::{DialogExt, FilePath};

    let result = app_handle.dialog().file().blocking_pick_folder();

    match result {
        Some(FilePath::Path(path)) => Ok(Some(path.to_string_lossy().to_string())),
        Some(FilePath::Url(url)) => Ok(Some(url.to_string())),
        None => Ok(None),
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tracing::info!("[STARTUP] Rust backend starting...");
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
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

            let settings_arc = Arc::new(settings_manager);

            let mut current_settings = settings_arc
                .get_settings()
                .unwrap_or_else(|_| core::settings::AppSettings::default());

            if current_settings.cloud.device_id.trim().is_empty() {
                current_settings.cloud.device_id = default_device_id();
                if let Err(err) = settings_arc.update_settings(current_settings.clone()) {
                    tracing::warn!("[SETTINGS] Failed to persist device id: {err}");
                }
            }

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

            // Cloud backend initialization
            let cloud_downloads_dir = app_data_dir.join("data").join("cloud_downloads");
            if let Err(e) = std::fs::create_dir_all(&cloud_downloads_dir) {
                tracing::warn!("[CLOUD] Failed to create cloud downloads dir: {e}");
            }
            let cloud: Box<dyn CloudBackend + Send> = Box::new(DisabledCloudBackend);

            let cloud_arc: CloudBackendState = Arc::new(Mutex::new(cloud));

            if let Err(err) = tauri::async_runtime::block_on(switch_cloud_backend(
                &app.handle(),
                &cloud_arc,
                settings_arc.clone(),
                current_settings.cloud_mode.clone(),
                current_settings.clone(),
            )) {
                tracing::error!("[CLOUD] Failed to initialize cloud backend: {err}");
            }

            tracing::info!(
                "{} Cloud backend initialized",
                log_tag(&current_settings.cloud_mode)
            );

            // Register state
            let history_arc = Arc::new(history_manager);

            let profiles_arc = Arc::new(RwLock::new(profile_manager));

            app.manage(WatcherManager::default());
            app.manage(history_arc.clone());
            app.manage(profiles_arc.clone());
            app.manage(settings_arc.clone());
            app.manage(cloud_arc.clone());

            // Initialize SyncManager
            let sync_manager = SyncManager::new(
                app.handle().clone(),
                cloud_arc,
                history_arc,
                profiles_arc,
                settings_arc,
            );

            app.manage(sync_manager.clone());

            // Start background tasks after all state is managed
            // Use tauri::async_runtime to spawn in Tauri's runtime context
            info!("[INIT] About to spawn SyncManager background task...");
            tauri::async_runtime::spawn(async move {
                info!("[INIT] Inside async block, calling start_background_task()");
                sync_manager.start_background_task();
                info!("[INIT] start_background_task() returned");
            });
            info!("[INIT] Background task spawned");

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            greet,
            select_directory,
            start_watcher,
            stop_watcher,
            list_profiles,
            get_profile,
            save_profile,
            delete_profile,
            package_save,
            package_game,
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
            check_path_status,
            open_folder,
            upload_cloud_save,
            list_cloud_versions,
            download_cloud_save,
            download_cloud_version,
            get_cloud_config,
            update_cloud_config,
            validate_official_cloud_settings,
            validate_self_host_settings,
            get_cloud_status,
            login_cloud,
            signup_cloud,
            logout_cloud,
            list_cloud_devices,
            register_cloud_device,
            remove_cloud_device,
            reconnect_cloud,
            update_cloud_mode,
            get_upload_url,
            notify_upload,
            get_sync_status,
            force_sync_now,
            clear_sync_queue,
            get_conflict_details,
            resolve_conflict_upload,
            resolve_conflict_download,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

pub(crate) async fn switch_cloud_backend(
    app: &tauri::AppHandle,
    cloud_state: &CloudBackendState,
    settings_manager: Arc<SettingsManager>,
    mode: CloudMode,
    settings: AppSettings,
) -> Result<(), CloudError> {
    let tag = log_tag(&mode);
    tracing::debug!("{tag} Switching backend to {:?}", mode);

    let backend: Box<dyn CloudBackend + Send> = match mode {
        CloudMode::Official => {
            tracing::info!("{tag} Preparing official cloud backend");
            Box::new(HttpCloudBackend::new(
                settings_manager.clone(),
                CloudMode::Official,
            )?)
        }
        CloudMode::SelfHost => {
            tracing::info!("{tag} Preparing self-hosted cloud backend");
            Box::new(HttpCloudBackend::new(
                settings_manager.clone(),
                CloudMode::SelfHost,
            )?)
        }
        CloudMode::Off => {
            tracing::info!("{tag} Disabling cloud backend");
            Box::new(DisabledCloudBackend)
        }
    };

    {
        let mut guard = cloud_state.lock().await;
        *guard = backend;
    }

    let _ = app.emit("cloud://backend-switched", &mode);
    tracing::info!(
        "{tag} Backend switched to {:?} (cloud enabled: {})",
        mode,
        settings.cloud.enabled
    );
    Ok(())
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

        let setup_marker = profiles_dir.join(".setup_complete");

        // Only create default profiles if setup hasn't completed yet
        if !setup_marker.exists() {
            // Create default RetroArch profile on first run
            if let Err(e) = std::fs::create_dir_all(&profiles_dir) {
                tracing::error!("[PROFILE] Failed to create profiles dir: {e}");
            } else {
                let retroarch_profile = profiles_dir.join("retroarch.json");
                if !retroarch_profile.exists() {
                    let default_profile = include_str!("../resources/profiles/retroarch.json");
                    let _ = std::fs::write(&retroarch_profile, default_profile);
                }

                let drastic_profile = profiles_dir.join("drastic.json");
                if !drastic_profile.exists() {
                    let default_profile = include_str!("../resources/profiles/drastic.json");
                    let _ = std::fs::write(&drastic_profile, default_profile);
                }

                let ppsspp_profile = profiles_dir.join("ppsspp.json");
                if !ppsspp_profile.exists() {
                    let default_profile = include_str!("../resources/profiles/ppsspp.json");
                    let _ = std::fs::write(&ppsspp_profile, default_profile);
                }

                let duckstation_profile = profiles_dir.join("duckstation.json");
                if !duckstation_profile.exists() {
                    let default_profile = include_str!("../resources/profiles/duckstation.json");
                    let _ = std::fs::write(&duckstation_profile, default_profile);
                }

                let aethersx2_profile = profiles_dir.join("aethersx2.json");
                if !aethersx2_profile.exists() {
                    let default_profile = include_str!("../resources/profiles/aethersx2.json");
                    let _ = std::fs::write(&aethersx2_profile, default_profile);
                }

                let dolphin_profile = profiles_dir.join("dolphin.json");
                if !dolphin_profile.exists() {
                    let default_profile = include_str!("../resources/profiles/dolphin.json");
                    let _ = std::fs::write(&dolphin_profile, default_profile);
                }

                // Mark setup as complete
                let _ = std::fs::write(&setup_marker, "setup complete");
            }
        }

        (profiles_dir.clone(), profiles_dir)
    }

    #[cfg(not(target_os = "android"))]
    {
        let default_dir = app
            .path()
            .resource_dir()
            .expect("failed to get resource dir")
            .join("resources")
            .join("profiles");
        let user_dir = app_data_dir.join("profiles");
        (default_dir, user_dir)
    }
}
