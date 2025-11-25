use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Mutex;

use reqwest::Client;
use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Emitter, Manager, State};

use crate::core::cloud::{
    log_tag, CloudBackend, CloudDevice, CloudError, CloudVersionSummary, UploadRequest,
    UploadUrlResponse,
};
use crate::core::history::HistoryManager;
use crate::core::settings::{CloudMode, CloudSettings, SelfHostSettings, SettingsManager};
use crate::core::sync::SyncManager;
use crate::switch_cloud_backend;

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

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct UploadMetadataPayload {
    pub size_bytes: u64,
    pub sha256: String,
    pub file_list: Vec<String>,
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
pub async fn get_upload_url(
    game_id: String,
    version_id: String,
    metadata: UploadMetadataPayload,
    cloud: State<'_, Arc<Mutex<Box<dyn CloudBackend + Send>>>>,
    settings: State<'_, Arc<SettingsManager>>,
) -> Result<UploadUrlResponse, String> {
    ensure_cloud_mode_enabled(&settings).map_err(cloud_error_to_string)?;

    let backend = cloud.lock().await;
    let device_id = backend.ensure_device_id().map_err(cloud_error_to_string)?;
    let payload = UploadRequest {
        game_id,
        version_id,
        size_bytes: metadata.size_bytes,
        sha256: metadata.sha256,
        file_list: metadata.file_list,
        device_id: Some(device_id),
    };

    backend
        .request_upload_url(payload)
        .await
        .map_err(cloud_error_to_string)
}

#[tauri::command]
pub async fn notify_upload(
    game_id: String,
    version_id: String,
    metadata: UploadMetadataPayload,
    cloud: State<'_, Arc<Mutex<Box<dyn CloudBackend + Send>>>>,
    settings: State<'_, Arc<SettingsManager>>,
) -> Result<(), String> {
    ensure_cloud_mode_enabled(&settings).map_err(cloud_error_to_string)?;

    let backend = cloud.lock().await;
    let device_id = backend.ensure_device_id().map_err(cloud_error_to_string)?;
    let payload = UploadRequest {
        game_id,
        version_id,
        size_bytes: metadata.size_bytes,
        sha256: metadata.sha256,
        file_list: metadata.file_list,
        device_id: Some(device_id),
    };

    backend
        .notify_upload_complete(payload)
        .await
        .map_err(cloud_error_to_string)
}

#[tauri::command]
pub async fn list_cloud_devices(
    app: AppHandle,
    cloud: State<'_, Arc<Mutex<Box<dyn CloudBackend + Send>>>>,
    settings: State<'_, Arc<SettingsManager>>,
) -> Result<Vec<CloudDevice>, String> {
    ensure_cloud_mode_enabled(&settings).map_err(cloud_error_to_string)?;

    let token = ensure_api_key(&settings)?;

    let backend = cloud.lock().await;
    match backend.list_devices(token).await {
        Ok(devices) => {
            let _ = app.emit("cloud://device-updated", devices.clone());
            Ok(devices)
        }
        Err(err) => {
            let message = cloud_error_to_string(err);
            let _ = app.emit("cloud://device-error", message.clone());
            Err(message)
        }
    }
}

#[tauri::command]
pub async fn register_cloud_device(
    app: AppHandle,
    device_id: String,
    platform: String,
    device_name: String,
    cloud: State<'_, Arc<Mutex<Box<dyn CloudBackend + Send>>>>,
    settings: State<'_, Arc<SettingsManager>>,
) -> Result<(), String> {
    ensure_cloud_mode_enabled(&settings).map_err(cloud_error_to_string)?;

    let token = ensure_api_key(&settings)?;
    let backend = cloud.lock().await;

    match backend
        .register_device(token.clone(), device_id.clone(), platform, device_name)
        .await
    {
        Ok(_) => {
            let devices = backend
                .list_devices(token)
                .await
                .map_err(cloud_error_to_string)?;
            let _ = app.emit("cloud://device-updated", devices);
            Ok(())
        }
        Err(err) => {
            let message = cloud_error_to_string(err);
            let _ = app.emit("cloud://device-error", message.clone());
            Err(message)
        }
    }
}

#[tauri::command]
pub async fn remove_cloud_device(
    app: AppHandle,
    device_id: String,
    cloud: State<'_, Arc<Mutex<Box<dyn CloudBackend + Send>>>>,
    settings: State<'_, Arc<SettingsManager>>,
) -> Result<(), String> {
    ensure_cloud_mode_enabled(&settings).map_err(cloud_error_to_string)?;

    let token = ensure_api_key(&settings)?;

    let backend = cloud.lock().await;
    match backend.remove_device(token.clone(), device_id).await {
        Ok(_) => {
            let devices = backend
                .list_devices(token)
                .await
                .map_err(cloud_error_to_string)?;
            let _ = app.emit("cloud://device-updated", devices);
            Ok(())
        }
        Err(err) => {
            let message = cloud_error_to_string(err);
            let _ = app.emit("cloud://device-error", message.clone());
            Err(message)
        }
    }
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

    if let Err(err) = ensure_config(&cloud, Some(&app)).await {
        let message = format!("Cloud not configured: {err}");
        let _ = app.emit(
            "sync://download-error",
            DownloadErrorPayload {
                version_id: version_id.clone(),
                message: message.clone(),
            },
        );
        return Err(message);
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
    sync: State<'_, SyncManager>,
) -> Result<CloudMode, String> {
    let parsed_mode = parse_cloud_mode(&new_mode)?;
    let tag = log_tag(&parsed_mode);
    tracing::info!("{tag} Requested cloud mode update to {:?}", parsed_mode);

    let mut app_settings = settings_manager
        .get_settings()
        .map_err(|e| format!("Failed to load settings: {e}"))?;
    app_settings.cloud_mode = parsed_mode.clone();
    let updated_settings = settings_manager
        .update_settings(app_settings)
        .map_err(|e| format!("Failed to save settings: {e}"))?;

    if parsed_mode == CloudMode::Official {
        sync.pause();
        let _ = app.emit("cloud://reconnect-started", "reconnect");
    } else {
        sync.resume();
    }

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

    let validation_result = match parsed_mode {
        CloudMode::Official => {
            let result = validate_official_config(&app, &updated_settings.cloud, false).await;
            match result {
                Ok(_) => {
                    sync.resume();
                    let _ = app.emit("cloud://online", "online");
                }
                Err(ref err) => {
                    let _ = app.emit("cloud://reconnect-required", err.clone());
                }
            }
            result
        }
        CloudMode::SelfHost => {
            validate_self_host_config(&app, &updated_settings.self_host, false).await
        }
        CloudMode::Off => Ok(()),
    };

    if let Err(err) = validation_result {
        return Err(err);
    }

    let _ = app.emit("cloud://mode-changed", &parsed_mode);
    tracing::info!("{tag} Cloud mode changed to {:?}", parsed_mode);
    Ok(parsed_mode)
}

#[tauri::command]
pub async fn reconnect_cloud(
    app: AppHandle,
    settings_manager: State<'_, Arc<SettingsManager>>,
    sync: State<'_, SyncManager>,
) -> Result<(), String> {
    let app_settings = settings_manager
        .get_settings()
        .map_err(|e| format!("Failed to load settings: {e}"))?;

    if app_settings.cloud_mode == CloudMode::Off {
        return Err(CloudError::Disabled.to_string());
    }

    if app_settings.cloud_mode != CloudMode::Official {
        return Err("Reconnect is only available in Official cloud mode".to_string());
    }

    sync.pause();
    let _ = app.emit("cloud://reconnect-started", "reconnect");

    let validation_result = validate_official_config(&app, &app_settings.cloud, false).await;
    match validation_result {
        Ok(_) => {
            sync.resume();
            let _ = app.emit("cloud://online", "online");
            Ok(())
        }
        Err(err) => {
            let _ = app.emit("cloud://reconnect-required", err.clone());
            Err(err)
        }
    }
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

    let base_url = match app_settings.cloud_mode {
        CloudMode::Official => app_settings.cloud.base_url.clone(),
        CloudMode::SelfHost => app_settings.self_host.api_server.clone(),
        CloudMode::Off => String::new(),
    }
    .trim_end_matches('/')
    .to_string();

    if base_url.is_empty() {
        let message = "Cloud base URL is missing".to_string();
        let _ = app.emit("sync://offline", message.clone());
        return Err(message);
    }

    let timeout = Duration::from_secs(app_settings.cloud.timeout_seconds.max(1));
    let client = Client::builder()
        .timeout(timeout)
        .build()
        .map_err(|e| format!("Failed to build client: {e}"))?;

    let mut connected = false;
    let mut last_error: Option<String> = None;
    for url in [format!("{}/health", base_url), base_url.clone()] {
        let mut request = client.get(url.clone());
        if app_settings.cloud_mode == CloudMode::Official && !app_settings.cloud.api_key.is_empty()
        {
            request = request.header(
                "Authorization",
                format!("Bearer {}", app_settings.cloud.api_key),
            );
        }
        if app_settings.cloud_mode == CloudMode::SelfHost
            && !app_settings.self_host.access_key.is_empty()
        {
            request = request.header(
                "Authorization",
                format!("Bearer {}", app_settings.self_host.access_key),
            );
        }

        match request.send().await {
            Ok(resp) => {
                if resp.status().is_success() {
                    connected = true;
                    break;
                }
                last_error = Some(format!("status {}", resp.status()));
            }
            Err(err) => {
                last_error = Some(err.to_string());
            }
        }
    }

    if connected {
        let _ = app.emit("sync://online", "online");
    } else {
        let _ = app.emit(
            "sync://offline",
            last_error.unwrap_or_else(|| "offline".to_string()),
        );
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

#[tauri::command]
pub async fn validate_official_cloud_settings(
    new_config: CloudSettings,
    app: AppHandle,
    settings_manager: State<'_, Arc<SettingsManager>>,
) -> Result<(), String> {
    let current_settings = settings_manager
        .get_settings()
        .map_err(|err| err.to_string())?;

    if current_settings.cloud_mode == CloudMode::Off {
        return Err(CloudError::Disabled.to_string());
    }

    validate_official_config(&app, &new_config, true).await
}

#[tauri::command]
pub async fn validate_self_host_settings(
    sh: SelfHostSettings,
    app: AppHandle,
    settings_manager: State<'_, Arc<SettingsManager>>,
) -> Result<(), String> {
    let current_settings = settings_manager
        .get_settings()
        .map_err(|err| err.to_string())?;

    if current_settings.cloud_mode == CloudMode::Off {
        return Err(CloudError::Disabled.to_string());
    }

    validate_self_host_config(&app, &sh, true).await
}

async fn validate_self_host_config(
    app: &AppHandle,
    settings: &SelfHostSettings,
    emit_validation_events: bool,
) -> Result<(), String> {
    let tag = log_tag(&CloudMode::SelfHost);
    tracing::debug!("{tag} Validating self-hosted cloud settings");

    let id_server = settings.id_server.trim();
    let relay_server = settings.relay_server.trim();
    let api_server = settings.api_server.trim();
    let access_key = settings.access_key.trim();

    if id_server.is_empty() {
        let message = "ID server URL is required for self-host mode".to_string();
        if emit_validation_events {
            emit_validation(app, CloudMode::SelfHost, false, message.clone());
        }
        return Err(message);
    }

    if relay_server.is_empty() {
        let message = "Relay server URL is required for self-host mode".to_string();
        if emit_validation_events {
            emit_validation(app, CloudMode::SelfHost, false, message.clone());
        }
        return Err(message);
    }

    if api_server.is_empty() {
        let message = "API server URL is required for self-host mode".to_string();
        if emit_validation_events {
            emit_validation(app, CloudMode::SelfHost, false, message.clone());
        }
        return Err(message);
    }

    if access_key.is_empty() {
        let message = "Access key is required for self-host mode".to_string();
        if emit_validation_events {
            emit_validation(app, CloudMode::SelfHost, false, message.clone());
        }
        return Err(message);
    }

    for url in [id_server, relay_server, api_server] {
        if !(url.starts_with("http://") || url.starts_with("https://")) {
            let message = "All URLs must start with http:// or https://".to_string();
            if emit_validation_events {
                emit_validation(app, CloudMode::SelfHost, false, message.clone());
            }
            return Err(message);
        }
    }

    let client = Client::builder()
        .timeout(Duration::from_secs(30))
        .build()
        .map_err(|e| format!("Failed to build client: {e}"))?;

    let base_api = api_server.trim_end_matches('/');
    let health_url = format!("{}/health", base_api);
    let version_url = format!("{}/version", base_api);

    let auth_header = format!("Bearer {}", access_key);
    let response = client
        .get(health_url.clone())
        .header("Authorization", auth_header.clone())
        .send()
        .await;

    let response = match response {
        Ok(resp) => Ok(resp),
        Err(_) => {
            client
                .get(version_url.clone())
                .header("Authorization", auth_header.clone())
                .send()
                .await
        }
    };

    let (valid, message) = match response {
        Ok(resp) if resp.status().is_success() => (true, "Self-host servers reachable".to_string()),
        Ok(resp) => (
            false,
            format!("Health check failed with status {}", resp.status()),
        ),
        Err(err) => (false, format!("Health check error: {err}")),
    };

    if emit_validation_events {
        emit_validation(app, CloudMode::SelfHost, valid, message.clone());
    }
    if valid {
        Ok(())
    } else {
        Err(message)
    }
}

async fn validate_official_config(
    app: &AppHandle,
    settings: &CloudSettings,
    emit_validation_events: bool,
) -> Result<(), String> {
    let tag = log_tag(&CloudMode::Official);
    tracing::debug!("{tag} Validating official cloud settings");

    let base_url = settings.base_url.trim();
    if base_url.is_empty() {
        let message = "Base URL is required for official cloud mode".to_string();
        if emit_validation_events {
            emit_validation(app, CloudMode::Official, false, message.clone());
        }
        return Err(message);
    }

    if settings.api_key.trim().is_empty() {
        let message = "API key is required for official cloud mode".to_string();
        if emit_validation_events {
            emit_validation(app, CloudMode::Official, false, message.clone());
        }
        return Err(message);
    }

    let timeout = Duration::from_secs(settings.timeout_seconds.max(1));
    let client = Client::builder()
        .timeout(timeout)
        .build()
        .map_err(|e| format!("Failed to build client: {e}"))?;

    let base = base_url.trim_end_matches('/');
    let health_url = format!("{}/health", base);
    let auth_header = format!("Bearer {}", settings.api_key.trim());
    let response = client
        .get(health_url.clone())
        .header("Authorization", auth_header.clone())
        .send()
        .await;

    let response = match response {
        Ok(resp) => Ok(resp),
        Err(_) => {
            client
                .get(base.to_string())
                .header("Authorization", auth_header)
                .send()
                .await
        }
    };

    let (valid, message) = match response {
        Ok(resp) if resp.status().is_success() => (true, "Official cloud reachable".to_string()),
        Ok(resp) => (
            false,
            format!("Connectivity failed with status {}", resp.status()),
        ),
        Err(err) => (false, format!("Connectivity error: {err}")),
    };

    if valid {
        let _ = app.emit("sync://online", "online");
    } else {
        let _ = app.emit("sync://offline", message.clone());
    }

    if emit_validation_events {
        emit_validation(app, CloudMode::Official, valid, message.clone());
    }
    if valid {
        Ok(())
    } else {
        Err(message)
    }
}

fn mode_to_str(mode: &CloudMode) -> &'static str {
    match mode {
        CloudMode::Official => "official",
        CloudMode::SelfHost => "self_host",
        CloudMode::Off => "off",
    }
}
