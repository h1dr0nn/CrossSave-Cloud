use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Mutex;

use reqwest::Client;
use serde::Serialize;
use tauri::{AppHandle, Emitter, Manager, State};

use crate::core::cloud::{CloudBackend, CloudDevice, CloudError, CloudVersionSummary};
use crate::core::history::HistoryManager;
use crate::core::settings::{CloudMode, CloudSettings, SettingsManager};
use crate::{mode_log_tag, switch_cloud_backend};

#[derive(Clone, Debug, Serialize)]
pub struct CloudStatus {
    pub enabled: bool,
    pub device_id: String,
    pub connected: bool,
}

#[derive(Clone, Debug, Serialize)]
pub struct LoginResult {
    pub token: String,
    pub device_id: String,
}

#[derive(Clone, Debug, Serialize)]
struct CloudValidationPayload {
    mode: String,
    message: String,
}

#[derive(Clone, Debug, Serialize)]
struct DownloadProgressPayload {
    version_id: String,
    progress: u8,
}

#[derive(Clone, Debug, Serialize)]
struct DownloadCompletePayload {
    version_id: String,
    path: String,
}

#[derive(Clone, Debug, Serialize)]
struct DownloadErrorPayload {
    version_id: String,
    message: String,
}

#[tauri::command]
pub async fn login_cloud(
    email: String,
    password: String,
    cloud: State<'_, Arc<Mutex<Box<dyn CloudBackend + Send>>>>,
    settings: State<'_, Arc<SettingsManager>>,
) -> Result<LoginResult, String> {
    ensure_cloud_mode_enabled(&settings).map_err(cloud_error_to_string)?;

    let backend = cloud.lock().await;
    let token = backend
        .login(email, password)
        .await
        .map_err(cloud_error_to_string)?;

    let device_id = backend.get_device_id().map_err(cloud_error_to_string)?;

    let mut app_settings = settings
        .get_settings()
        .map_err(|e| format!("Failed to load settings: {e}"))?;
    app_settings.cloud.enabled = true;
    app_settings.cloud.api_key = token.clone();
    app_settings.cloud.device_id = device_id.clone();
    settings
        .update_settings(app_settings)
        .map_err(|e| format!("Failed to persist settings: {e}"))?;

    Ok(LoginResult { token, device_id })
}

#[tauri::command]
pub async fn logout_cloud(settings: State<'_, Arc<SettingsManager>>) -> Result<(), String> {
    ensure_cloud_mode_enabled(&settings).map_err(cloud_error_to_string)?;

    let mut app_settings = settings
        .get_settings()
        .map_err(|e| format!("Failed to load settings: {e}"))?;
    app_settings.cloud.enabled = false;
    app_settings.cloud.api_key.clear();
    settings
        .update_settings(app_settings)
        .map_err(|e| format!("Failed to persist settings: {e}"))?;
    Ok(())
}

/// Uploads a specific local save version to the cloud.
///
/// This function:
/// 1. Retrieves the local history entry for the given version.
/// 2. Converts the archive path to a `PathBuf`.
/// 3. Spawns an async task to upload the archive using the configured backend.
/// 4. Returns a summary of the uploaded version.
#[tauri::command]
pub async fn upload_cloud_save(
    game_id: String,
    _emulator_id: String,
    local_version_id: String,
    cloud: State<'_, Arc<Mutex<Box<dyn CloudBackend + Send>>>>,
    history: State<'_, Arc<HistoryManager>>,
    settings: State<'_, Arc<SettingsManager>>,
) -> Result<CloudVersionSummary, String> {
    ensure_cloud_mode_enabled(&settings).map_err(cloud_error_to_string)?;

    let history_entry = history
        .get_history_item(game_id, local_version_id)
        .map_err(|e| format!("Failed to find local version: {e}"))?;

    let archive_path_buf = PathBuf::from(&history_entry.archive_path);
    let metadata = history_entry.metadata;

    let backend = cloud.lock().await;
    backend
        .upload_archive(metadata, archive_path_buf)
        .await
        .map_err(cloud_error_to_string)
}

#[tauri::command]
pub async fn list_devices(
    cloud: State<'_, Arc<Mutex<Box<dyn CloudBackend + Send>>>>,
    settings: State<'_, Arc<SettingsManager>>,
) -> Result<Vec<CloudDevice>, String> {
    ensure_cloud_mode_enabled(&settings).map_err(cloud_error_to_string)?;

    let token = ensure_api_key(&settings)?;

    let backend = cloud.lock().await;
    backend
        .list_devices(token)
        .await
        .map_err(cloud_error_to_string)
}

#[tauri::command]
pub async fn remove_device(
    device_id: String,
    cloud: State<'_, Arc<Mutex<Box<dyn CloudBackend + Send>>>>,
    settings: State<'_, Arc<SettingsManager>>,
) -> Result<(), String> {
    ensure_cloud_mode_enabled(&settings).map_err(cloud_error_to_string)?;

    let token = ensure_api_key(&settings)?;

    let backend = cloud.lock().await;
    backend
        .remove_device(token, device_id)
        .await
        .map_err(cloud_error_to_string)
}

/// Lists available save versions for a game from the cloud.
///
/// Returns a list of `CloudVersionSummary` objects, optionally limited by `limit`.
#[tauri::command]
pub async fn list_cloud_versions(
    game_id: String,
    limit: Option<u32>,
    cloud: State<'_, Arc<Mutex<Box<dyn CloudBackend + Send>>>>,
    settings: State<'_, Arc<SettingsManager>>,
) -> Result<Vec<CloudVersionSummary>, String> {
    ensure_cloud_mode_enabled(&settings).map_err(cloud_error_to_string)?;

    if ensure_config(&cloud, None).await.is_err() {
        return Err("Cloud not configured".into());
    }
    let backend = cloud.lock().await;
    backend
        .list_versions(game_id, limit.map(|l| l as usize))
        .await
        .map_err(cloud_error_to_string)
}

/// Downloads a specific version from the cloud to the local downloads directory.
///
/// The file is saved to `AppData/data/cloud_downloads/{game_id}_{version_id}.zip`.
/// Returns the absolute path to the downloaded file.
#[tauri::command]
pub async fn download_cloud_version(
    game_id: String,
    version_id: String,
    cloud: State<'_, Arc<Mutex<Box<dyn CloudBackend + Send>>>>,
    settings: State<'_, Arc<SettingsManager>>,
    app: tauri::AppHandle,
) -> Result<String, String> {
    ensure_cloud_mode_enabled(&settings).map_err(cloud_error_to_string)?;

    let app_data_dir = app
        .path()
        .app_data_dir()
        .map_err(|e| format!("Failed to get app data dir: {e}"))?;
    let downloads_dir = app_data_dir.join("data").join("cloud_downloads");
    let target_path = downloads_dir.join(format!("{}_{}.zip", game_id, version_id));

    if ensure_config(&cloud, Some(&app)).await.is_err() {
        return Err("Cloud not configured".into());
    }

    let backend = cloud.lock().await;
    let start_payload = DownloadProgressPayload {
        version_id: version_id.clone(),
        progress: 0,
    };
    let _ = app.emit("sync://download-progress", start_payload);

    match backend
        .download_version(game_id, version_id.clone(), target_path.clone())
        .await
    {
        Ok(_) => {
            let _ = app.emit(
                "sync://download-progress",
                DownloadProgressPayload {
                    version_id: version_id.clone(),
                    progress: 100,
                },
            );
            let _ = app.emit(
                "sync://download-complete",
                DownloadCompletePayload {
                    version_id: version_id.clone(),
                    path: target_path.to_string_lossy().to_string(),
                },
            );
            Ok(target_path.to_string_lossy().to_string())
        }
        Err(err) => {
            let err_msg = cloud_error_to_string(err);
            let _ = app.emit(
                "sync://download-error",
                DownloadErrorPayload {
                    version_id: version_id.clone(),
                    message: err_msg.clone(),
                },
            );
            Err(err_msg)
        }
    }
}

/// Retrieves the current cloud configuration settings.
#[tauri::command]
pub async fn get_cloud_config(
    settings: State<'_, Arc<SettingsManager>>,
) -> Result<CloudSettings, String> {
    ensure_cloud_mode_enabled(&settings).map_err(cloud_error_to_string)?;

    settings
        .get_settings()
        .map(|s| s.cloud)
        .map_err(|e| format!("Failed to load settings: {e}"))
}

/// Updates the cloud configuration settings.
///
/// Persists the new settings to the application's settings file.
#[tauri::command]
pub async fn update_cloud_config(
    new_config: CloudSettings,
    settings: State<'_, Arc<SettingsManager>>,
) -> Result<CloudSettings, String> {
    ensure_cloud_mode_enabled(&settings).map_err(cloud_error_to_string)?;

    let mut app_settings = settings
        .get_settings()
        .map_err(|e| format!("Failed to load settings: {e}"))?;
    app_settings.cloud = new_config;
    settings
        .update_settings(app_settings)
        .map(|s| s.cloud)
        .map_err(|e| format!("Failed to save settings: {e}"))
}

#[tauri::command]
pub async fn update_cloud_mode(
    new_mode: String,
    app: AppHandle,
    cloud: State<'_, Arc<Mutex<Box<dyn CloudBackend + Send>>>>,
    settings_manager: State<'_, Arc<SettingsManager>>,
) -> Result<CloudMode, String> {
    let parsed_mode = parse_cloud_mode(&new_mode)?;
    let tag = mode_log_tag(&parsed_mode);
    tracing::info!("{tag} Requested cloud mode update to {:?}", parsed_mode);

    let mut app_settings = settings_manager
        .get_settings()
        .map_err(|e| format!("Failed to load settings: {e}"))?;
    app_settings.cloud_mode = parsed_mode.clone();
    let updated_settings = settings_manager
        .update_settings(app_settings)
        .map_err(|e| format!("Failed to save settings: {e}"))?;

    if let Err(err) = switch_cloud_backend(
        &app,
        &cloud,
        (*settings_manager).clone(),
        parsed_mode.clone(),
        updated_settings.clone(),
    )
    .await
    {
        return Err(cloud_error_to_string(err));
    }

    match parsed_mode {
        CloudMode::Official => validate_official_cloud_settings(&app, &updated_settings.cloud)?,
        CloudMode::SelfHost => validate_self_host_settings(&app, &updated_settings.cloud)?,
        CloudMode::Off => emit_validation(&app, CloudMode::Off, true, "Cloud mode disabled".into()),
    };

    let _ = app.emit("cloud://mode-changed", &parsed_mode);
    tracing::info!("{tag} Cloud mode changed to {:?}", parsed_mode);
    Ok(parsed_mode)
}

/// Checks the status of the cloud connection by performing a HEAD request to `/ping`.
#[tauri::command]
pub async fn get_cloud_status(
    cloud: State<'_, Arc<Mutex<Box<dyn CloudBackend + Send>>>>,
    settings: State<'_, Arc<SettingsManager>>,
    app: tauri::AppHandle,
) -> Result<CloudStatus, String> {
    ensure_cloud_mode_enabled(&settings).map_err(cloud_error_to_string)?;

    let app_settings = settings
        .get_settings()
        .map_err(|e| format!("Failed to load settings: {e}"))?;

    let base_url = app_settings
        .cloud
        .base_url
        .trim_end_matches('/')
        .to_string();
    let ping_url = format!("{}/ping", base_url);
    let timeout = Duration::from_secs(app_settings.cloud.timeout_seconds.max(1));
    let client = Client::builder()
        .timeout(timeout)
        .build()
        .map_err(|e| format!("Failed to build client: {e}"))?;

    let mut connected = false;
    if let Ok(response) = client.head(ping_url).send().await {
        connected = response.status().is_success();
    }

    if !connected {
        let _ = app.emit("sync://offline", "offline");
    }

    let backend = cloud.lock().await;
    let device_id = backend.ensure_device_id().map_err(cloud_error_to_string)?;

    Ok(CloudStatus {
        enabled: app_settings.cloud.enabled,
        device_id,
        connected,
    })
}

async fn ensure_config(
    cloud: &State<'_, Arc<Mutex<Box<dyn CloudBackend + Send>>>>,
    app: Option<&tauri::AppHandle>,
) -> Result<(), String> {
    let backend = cloud.lock().await;
    if let Err(err) = backend.get_device_id() {
        if let Some(app) = app {
            let _ = app.emit("sync://offline", format!("config error: {err}"));
        }
        return Err(cloud_error_to_string(err));
    }
    Ok(())
}

fn ensure_api_key(settings: &State<'_, Arc<SettingsManager>>) -> Result<String, String> {
    let token = settings
        .get_settings()
        .map_err(|e| format!("Failed to load settings: {e}"))?
        .cloud
        .api_key;

    if token.is_empty() {
        return Err("Not logged in".into());
    }
    Ok(token)
}

fn cloud_error_to_string(error: CloudError) -> String {
    match error {
        CloudError::NotEnabled => "Cloud sync is not enabled".to_string(),
        CloudError::Disabled => "Cloud backend is disabled".to_string(),
        CloudError::NetworkError(msg) => format!("Network error: {}", msg),
        CloudError::StorageError(msg) => format!("Storage error: {}", msg),
        CloudError::NotFound(msg) => format!("Not found: {}", msg),
        CloudError::InvalidConfig(msg) => format!("Invalid configuration: {}", msg),
        CloudError::Io(msg) => format!("IO error: {}", msg),
        CloudError::Serialization(msg) => format!("Serialization error: {}", msg),
        CloudError::Unauthorized(msg) => format!("Unauthorized: {}", msg),
    }
}

fn ensure_cloud_mode_enabled(settings: &State<'_, Arc<SettingsManager>>) -> Result<(), CloudError> {
    let mode = settings
        .get_settings()
        .map_err(|e| CloudError::InvalidConfig(format!("Failed to load settings: {e}")))?
        .cloud_mode;

    if mode == CloudMode::Off {
        return Err(CloudError::Disabled);
    }

    Ok(())
}

fn parse_cloud_mode(new_mode: &str) -> Result<CloudMode, String> {
    match new_mode.to_lowercase().as_str() {
        "official" => Ok(CloudMode::Official),
        "selfhost" | "self_host" | "self-host" => Ok(CloudMode::SelfHost),
        "off" => Ok(CloudMode::Off),
        other => Err(format!("Unsupported cloud mode: {other}")),
    }
}

fn emit_validation(app: &AppHandle, mode: CloudMode, valid: bool, message: String) {
    let event = if valid {
        "cloud://config-valid"
    } else {
        "cloud://config-invalid"
    };

    let payload = CloudValidationPayload {
        mode: mode_to_str(&mode).to_string(),
        message: message.clone(),
    };

    let _ = app.emit(event, payload);
}

pub(crate) fn validate_self_host_settings(
    app: &AppHandle,
    settings: &CloudSettings,
) -> Result<(), String> {
    let tag = mode_log_tag(&CloudMode::SelfHost);
    tracing::debug!("{tag} Validating self-hosted cloud settings");

    let base_url = settings.base_url.trim();
    if base_url.is_empty() {
        let message = "Base URL is required for self-host mode".to_string();
        emit_validation(app, CloudMode::SelfHost, false, message.clone());
        return Err(message);
    }

    if !(base_url.starts_with("http://") || base_url.starts_with("https://")) {
        let message = "Base URL must start with http:// or https://".to_string();
        emit_validation(app, CloudMode::SelfHost, false, message.clone());
        return Err(message);
    }

    if settings.enabled && settings.api_key.trim().is_empty() {
        let message = "Access key is required for authenticated self-host mode".to_string();
        emit_validation(app, CloudMode::SelfHost, false, message.clone());
        return Err(message);
    }

    emit_validation(
        app,
        CloudMode::SelfHost,
        true,
        "Self-host configuration is valid".to_string(),
    );
    Ok(())
}

pub(crate) fn validate_official_cloud_settings(
    app: &AppHandle,
    settings: &CloudSettings,
) -> Result<(), String> {
    let tag = mode_log_tag(&CloudMode::Official);
    tracing::debug!("{tag} Validating official cloud settings");

    if settings.base_url.trim().is_empty() {
        let message = "Base URL is required for official cloud mode".to_string();
        emit_validation(app, CloudMode::Official, false, message.clone());
        return Err(message);
    }

    if settings.api_key.trim().is_empty() {
        let message = "API key is required for official cloud mode".to_string();
        emit_validation(app, CloudMode::Official, false, message.clone());
        return Err(message);
    }

    emit_validation(
        app,
        CloudMode::Official,
        true,
        "Official cloud configuration is valid".to_string(),
    );
    Ok(())
}

fn mode_to_str(mode: &CloudMode) -> &'static str {
    match mode {
        CloudMode::Official => "official",
        CloudMode::SelfHost => "self_host",
        CloudMode::Off => "off",
    }
}
