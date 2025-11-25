use std::collections::VecDeque;
use std::fs;
use std::path::PathBuf;
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc, RwLock,
};
use std::time::Duration;

use chrono::{DateTime, Utc};
use reqwest::{header::CONTENT_LENGTH, header::CONTENT_TYPE, Client};
use serde::{Deserialize, Serialize};
use serde_json::json;
use sha2::{Digest, Sha256};
use tauri::{AppHandle, Emitter, Manager};
use tokio::io::AsyncWriteExt;
use tokio::sync::{Mutex, Notify};
use tokio::time::sleep;
use tracing::{debug, error, info, warn};

use crate::core::cloud::{
    ensure_device_identity, log_tag, CloudBackend, CloudVersionSummary, DownloadUrlResponse,
    UploadRequest, UploadUrlResponse,
};
use crate::core::history::{HistoryEntry, HistoryManager};
use crate::core::packager::SaveMetadata;
use crate::core::profile::ProfileManager;
use crate::core::settings::{CloudMode, SettingsManager};
use zip::ZipArchive;

// ============================================================================
// Sync Decision Logic
// ============================================================================

#[derive(Debug, Clone, PartialEq)]
pub enum SyncDecision {
    Upload,
    Download(String), // version_id
    Conflict,
    Noop,
}

#[derive(Debug, Clone, Serialize)]
pub struct ConnectionStatus {
    pub connected: bool,
    pub last_success: Option<u64>, // timestamp in seconds
    pub last_error: Option<String>,
}

/// Determines the sync action based on local and cloud state.
///
/// Rules:
/// - If cloud has no versions -> Upload local (if exists)
/// - If local has no history -> Download latest cloud
/// - If both exist:
///   - If hashes match -> Noop
///   - If |timestamp difference| <= 2s -> Conflict
///   - Otherwise latest timestamp wins (upload if local is newer, download if cloud is newer)
pub fn determine_sync_action(
    local_entry: Option<&HistoryEntry>,
    cloud_versions: &[CloudVersionSummary],
    _current_device: &str,
) -> SyncDecision {
    let latest_cloud = cloud_versions.iter().max_by_key(|v| v.timestamp);

    match (local_entry, latest_cloud) {
        (Some(_), None) => SyncDecision::Upload,
        (None, Some(cloud)) => SyncDecision::Download(cloud.version_id.clone()),
        (None, None) => SyncDecision::Noop,
        (Some(local), Some(cloud)) => {
            let local_time = local.metadata.timestamp as u64;
            let cloud_time = cloud.timestamp;

            if cloud.sha256 == local.metadata.hash {
                return SyncDecision::Noop;
            }

            let time_diff = if cloud_time > local_time {
                cloud_time - local_time
            } else {
                local_time - cloud_time
            };

            let conflict_window = time_diff <= 2;

            if conflict_window {
                return SyncDecision::Conflict;
            }

            if cloud_time > local_time {
                SyncDecision::Download(cloud.version_id.clone())
            } else {
                SyncDecision::Upload
            }
        }
    }
}

async fn ensure_registered_device_for_sync(
    cloud: &Arc<Mutex<Box<dyn CloudBackend + Send>>>,
    settings: &Arc<SettingsManager>,
    app_handle: &AppHandle,
) -> Result<String, String> {
    let settings_snapshot = settings
        .get_settings()
        .map_err(|e| format!("Failed to load settings: {e}"))?;

    if settings_snapshot.cloud_mode == CloudMode::Off {
        return Err("Cloud sync is disabled".to_string());
    }

    let token = match settings_snapshot.cloud_mode {
        CloudMode::SelfHost => settings_snapshot.self_host.access_key.clone(),
        _ => settings_snapshot.cloud.api_key.clone(),
    };

    if token.trim().is_empty() {
        return Err("Cloud sync is not configured.".to_string());
    }

    let (device_id, platform, device_name) =
        ensure_device_identity(settings).map_err(|e| e.to_string())?;

    {
        let backend = cloud.lock().await;
        let devices = backend
            .list_devices(token.clone())
            .await
            .map_err(|e| e.to_string())?;
        if devices.iter().any(|device| device.device_id == device_id) {
            return Ok(device_id);
        }
    }

    let _ = app_handle.emit("sync://device-missing", device_id.clone());

    {
        let backend = cloud.lock().await;
        backend
            .register_device(
                token.clone(),
                device_id.clone(),
                platform.clone(),
                device_name.clone(),
            )
            .await
            .map_err(|e| {
                let message = e.to_string();
                let _ = app_handle.emit("sync://device-error", message.clone());
                message
            })?;
    }

    {
        let backend = cloud.lock().await;
        let devices = backend
            .list_devices(token)
            .await
            .map_err(|e| e.to_string())?;
        if devices.iter().any(|device| device.device_id == device_id) {
            let _ = app_handle.emit("cloud://device-registered", device_id.clone());
            return Ok(device_id);
        }
    }

    let message = "Device registration verification failed".to_string();
    let _ = app_handle.emit("sync://device-error", message.clone());
    Err(message)
}

#[derive(Clone, Debug, Serialize)]
struct DownloadProgressPayload {
    version_id: String,
    received_bytes: u64,
    total_bytes: u64,
}

#[derive(Clone, Debug, Serialize)]
struct DownloadCompletePayload {
    version_id: String,
}

#[derive(Clone, Debug, Serialize)]
struct DownloadErrorPayload {
    version_id: String,
    stage: String,
    message: String,
}

#[derive(Clone, Debug, Serialize)]
struct UploadProgressPayload {
    version_id: String,
    progress: u8,
}

#[derive(Clone, Debug, Serialize)]
struct UploadCompletePayload {
    version_id: String,
}

#[derive(Clone, Debug, Serialize)]
struct UploadErrorPayload {
    version_id: String,
    stage: String,
    reason: String,
    message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    status: Option<u16>,
}

// ...

// ============================================================================
// Upload Queue
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum UploadStatus {
    Pending,
    Uploading,
    Failed,
    Completed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UploadJob {
    pub game_id: String,
    pub emulator_id: String,
    pub version_id: String,
    pub archive_path: PathBuf,
    pub metadata: crate::core::packager::SaveMetadata,
    pub created_at: DateTime<Utc>,
    pub retries: u32,
    pub status: UploadStatus,
    pub total_size: u64,
    pub hash: String,
}

#[derive(Clone, Serialize)]
pub struct SyncStatus {
    pub queue_length: usize,
    pub active_job: Option<UploadJob>,
    pub last_sync: Option<DateTime<Utc>>,
    pub is_syncing: bool,
}

pub struct UploadQueue {
    queue: Arc<Mutex<VecDeque<UploadJob>>>,
    active_job: Arc<Mutex<Option<UploadJob>>>,
    notify: Arc<Notify>,
    online_notify: Arc<Notify>,
    online_status: Arc<AtomicBool>,
    app_handle: AppHandle,
    queue_path: PathBuf,
}

impl UploadQueue {
    pub fn new(app_handle: AppHandle, online_status: Arc<AtomicBool>) -> Self {
        let queue_path = app_handle
            .path()
            .app_data_dir()
            .map(|dir| dir.join("data").join("sync_queue.json"))
            .unwrap_or_else(|_| PathBuf::from("sync_queue.json"));
        Self {
            queue: Arc::new(Mutex::new(VecDeque::new())),
            active_job: Arc::new(Mutex::new(None)),
            notify: Arc::new(Notify::new()),
            online_notify: Arc::new(Notify::new()),
            online_status,
            app_handle,
            queue_path,
        }
    }

    pub async fn add_job(&self, job: UploadJob) {
        let should_notify = {
            let mut q = self.queue.lock().await;
            // Avoid duplicates
            if !q.iter().any(|j| j.version_id == job.version_id) {
                q.push_back(job.clone());
                info!("[QUEUE] Added job for game_id={}, queue length={}", job.game_id, q.len());
                true // Will notify after dropping lock
            } else {
                warn!("[QUEUE] Duplicate job detected for version_id={}, skipping", job.version_id);
                false
            }
        }; // Lock dropped here
        
        if should_notify {
            self.notify.notify_one(); // Notify AFTER lock is dropped
            self.emit_status().await;
            self.save_to_disk().await;
        }
    }

    pub async fn load_from_disk(&self) {
        if !self.queue_path.exists() {
            return;
        }

        if let Ok(content) = fs::read_to_string(&self.queue_path) {
            if let Ok(jobs) = serde_json::from_str::<Vec<UploadJob>>(&content) {
                let mut q = self.queue.lock().await;
                q.clear();
                let mut added = false;
                for mut job in jobs {
                    if job.status == UploadStatus::Uploading {
                        job.status = UploadStatus::Pending;
                    }
                    q.push_back(job);
                    added = true;
                }
                if added {
                    self.notify.notify_one();
                }
            }
        }
    }

    pub async fn save_to_disk(&self) {
        if let Some(parent) = self.queue_path.parent() {
            let _ = fs::create_dir_all(parent);
        }
        let q = self.queue.lock().await;
        // Only save pending and uploading jobs, not completed or failed
        let jobs_to_save: Vec<UploadJob> = q
            .iter()
            .filter(|job| {
                matches!(
                    job.status,
                    UploadStatus::Pending | UploadStatus::Uploading
                )
            })
            .cloned()
            .collect();

        if let Err(e) = std::fs::write(
            &self.queue_path,
            serde_json::to_string_pretty(&jobs_to_save).unwrap_or_default(),
        ) {
            warn!("[QUEUE] Failed to save queue to disk: {}", e);
        }
    }

    pub async fn get_status(&self) -> SyncStatus {
        let q_len = self.queue.lock().await.len();
        let active = self.active_job.lock().await.clone();

        let is_syncing = q_len > 0 || active.is_some();

        SyncStatus {
            queue_length: q_len,
            active_job: active,
            last_sync: Some(Utc::now()), // TODO: Store actual last sync time
            is_syncing,
        }
    }

    pub async fn clear(&self) {
        let mut q = self.queue.lock().await;
        q.clear();
        self.emit_status().await;
    }

    pub fn signal_online(&self) {
        self.online_notify.notify_waiters();
    }

    async fn wait_for_online(&self) {
        while !self.online_status.load(Ordering::SeqCst) {
            let _ = self.online_notify.notified().await;
        }
    }

    async fn emit_status(&self) {
        let status = self.get_status().await;
        let _ = self.app_handle.emit("sync://status", status);
    }

    pub async fn process_queue(
        &self,
        cloud: Arc<Mutex<Box<dyn CloudBackend + Send>>>,
        settings: Arc<SettingsManager>,
    ) {
        info!("[QUEUE] Queue processor started");
        loop {
            debug!("[QUEUE] Loop iteration start");
            if !self.online_status.load(Ordering::SeqCst) {
                debug!("[QUEUE] Waiting for online status...");
                self.wait_for_online().await;
                continue;
            }
            debug!("[QUEUE] Online status OK, proceeding to check queue");

            // Wait for a job
            let job = {
                let mut q = self.queue.lock().await;
                let queue_len = q.len();
                debug!("[QUEUE] Acquired queue lock, queue length={}", queue_len);
                if q.is_empty() {
                    None
                } else {
                    let job = q.pop_front();
                    if job.is_some() {
                        info!("[QUEUE] Popped job from queue, {} jobs remaining", q.len());
                    }
                    job
                }
            };

            if let Some(mut job) = job {
                info!("[QUEUE] Processing upload job for game_id={}", job.game_id);
                // Set active
                {
                    let mut active = self.active_job.lock().await;
                    *active = Some(job.clone());
                }
                self.emit_status().await;

                let mode = settings
                    .get_settings()
                    .map(|s| s.cloud_mode)
                    .unwrap_or(crate::core::settings::CloudMode::Off);
                let tag = log_tag(&mode);

                info!("{} [SYNC] Starting upload for {}", tag, job.game_id);

                job.status = UploadStatus::Uploading;
                self.save_to_disk().await;

                // Perform Upload
                let result = self.perform_upload(&job, &cloud, &settings).await;

                match result {
                    Ok(_) => {
                        info!("{} [SYNC] Upload complete for {}", tag, job.game_id);
                        job.status = UploadStatus::Completed;
                    }
                    Err(e) => {
                        error!("{} [SYNC] Upload failed for {}: {}", tag, job.game_id, e);
                        if job.retries < 3 {
                            job.retries += 1;
                            let retries = job.retries;
                            warn!("{} [SYNC] Retrying job (attempt {})", tag, retries);
                            let mut q = self.queue.lock().await;
                            q.push_front(job); // Put back at front
                            sleep(Duration::from_secs(2u64.pow(retries))).await;
                        // Exponential backoff
                        } else {
                            error!("{} [SYNC] Job failed after 3 retries, dropping.", tag);
                            job.status = UploadStatus::Failed;
                        }
                    }
                }

                self.save_to_disk().await;

                // Clear active
                {
                    let mut active = self.active_job.lock().await;
                    *active = None;
                }
                self.emit_status().await;
            } else {
                // Queue is empty, wait for notification
                debug!("[QUEUE] Queue empty, waiting for notification...");
                tokio::select! {
                    _ = self.notify.notified() => {
                        debug!("[QUEUE] Woke up from notify signal");
                    },
                    _ = self.online_notify.notified() => {
                        debug!("[QUEUE] Woke up from online signal");
                    },
                }
                debug!("[QUEUE] Notification received, looping back to check queue");
            }
        }
    }

    async fn perform_upload(
        &self,
        job: &UploadJob,
        cloud: &Arc<Mutex<Box<dyn CloudBackend + Send>>>,
        settings: &Arc<SettingsManager>,
    ) -> Result<(), String> {
        let emit_error = |payload: UploadErrorPayload, app_handle: &AppHandle| {
            let _ = app_handle.emit("sync://upload-error", payload.clone());
            payload.message
        };

        let settings_snapshot = settings.get_settings().map_err(|e| {
            emit_error(
                UploadErrorPayload {
                    version_id: job.version_id.clone(),
                    stage: "preflight".to_string(),
                    reason: "missing_config".to_string(),
                    message: e.to_string(),
                    status: None,
                },
                &self.app_handle,
            )
        })?;

        let base_url = match settings_snapshot.cloud_mode {
            crate::core::settings::CloudMode::SelfHost => {
                settings_snapshot.self_host.api_server.clone()
            }
            _ => settings_snapshot.cloud.base_url.clone(),
        };

        if settings_snapshot.cloud_mode == crate::core::settings::CloudMode::Off
            || base_url.trim().is_empty()
        {
            return Err(emit_error(
                UploadErrorPayload {
                    version_id: job.version_id.clone(),
                    stage: "preflight".to_string(),
                    reason: "missing_config".to_string(),
                    message: "Cloud sync is not configured.".to_string(),
                    status: None,
                },
                &self.app_handle,
            ));
        }

        let token_missing = match settings_snapshot.cloud_mode {
            crate::core::settings::CloudMode::SelfHost => {
                settings_snapshot.self_host.access_key.trim().is_empty()
            }
            _ => settings_snapshot.cloud.api_key.trim().is_empty(),
        };

        if token_missing {
            return Err(emit_error(
                UploadErrorPayload {
                    version_id: job.version_id.clone(),
                    stage: "preflight".to_string(),
                    reason: "missing_config".to_string(),
                    message: "Cloud sync is not configured.".to_string(),
                    status: None,
                },
                &self.app_handle,
            ));
        }

        let hash = calculate_sha256(&job.archive_path).map_err(|e| {
            emit_error(
                UploadErrorPayload {
                    version_id: job.version_id.clone(),
                    stage: "preflight".to_string(),
                    reason: "io".to_string(),
                    message: e.to_string(),
                    status: None,
                },
                &self.app_handle,
            )
        })?;
        let size_bytes = fs::metadata(&job.archive_path)
            .map_err(|e| {
                emit_error(
                    UploadErrorPayload {
                        version_id: job.version_id.clone(),
                        stage: "preflight".to_string(),
                        reason: "io".to_string(),
                        message: e.to_string(),
                        status: None,
                    },
                    &self.app_handle,
                )
            })?
            .len();

        let device_id = ensure_registered_device_for_sync(cloud, settings, &self.app_handle)
            .await
            .map_err(|message| {
                emit_error(
                    UploadErrorPayload {
                        version_id: job.version_id.clone(),
                        stage: "preflight".to_string(),
                        reason: "missing_config".to_string(),
                        message: message.clone(),
                        status: None,
                    },
                    &self.app_handle,
                )
            })?;

        let mut payload = UploadRequest {
            game_id: job.game_id.clone(),
            version_id: job.version_id.clone(),
            size_bytes,
            sha256: hash.clone(),
            file_list: job.metadata.file_list.clone(),
            emulator_id: Some(job.metadata.emulator_id.clone()),
            device_id: Some(device_id),
            worker_token: None,
        };

        let start_progress = UploadProgressPayload {
            version_id: job.version_id.clone(),
            progress: 0,
        };
        let _ = self
            .app_handle
            .emit("sync://upload-progress", start_progress.clone());

        let signed: UploadUrlResponse = {
            let backend = cloud.lock().await;
            match backend.request_upload_url(payload.clone()).await {
                Ok(url) => url,
                Err(err) => {
                    return Err(emit_error(
                        UploadErrorPayload {
                            version_id: payload.version_id.clone(),
                            stage: "request_url".to_string(),
                            reason: "backend_error".to_string(),
                            message: err.to_string(),
                            status: None,
                        },
                        &self.app_handle,
                    ));
                }
            }
        };

        payload.worker_token = signed.worker_token.clone();

        let archive_bytes = fs::read(&job.archive_path).map_err(|e| {
            emit_error(
                UploadErrorPayload {
                    version_id: job.version_id.clone(),
                    stage: "upload".to_string(),
                    reason: "io".to_string(),
                    message: e.to_string(),
                    status: None,
                },
                &self.app_handle,
            )
        })?;
        let client = Client::new();
        let put_resp = client
            .put(&signed.upload_url)
            .header(CONTENT_TYPE, "application/zip")
            .header(CONTENT_LENGTH, archive_bytes.len() as u64)
            .body(archive_bytes)
            .send()
            .await;

        let put_resp = match put_resp {
            Ok(resp) => resp,
            Err(err) => {
                return Err(emit_error(
                    UploadErrorPayload {
                        version_id: payload.version_id.clone(),
                        stage: "upload".to_string(),
                        reason: "transport_error".to_string(),
                        message: err.to_string(),
                        status: None,
                    },
                    &self.app_handle,
                ));
            }
        };

        if !put_resp.status().is_success() {
            let status = put_resp.status().as_u16();
            let message = format!("upload failed: {}", put_resp.status());
            return Err(emit_error(
                UploadErrorPayload {
                    version_id: payload.version_id.clone(),
                    stage: "upload".to_string(),
                    reason: "http_status".to_string(),
                    message,
                    status: Some(status),
                },
                &self.app_handle,
            ));
        }

        let mid_progress = UploadProgressPayload {
            version_id: job.version_id.clone(),
            progress: 80,
        };
        let _ = self
            .app_handle
            .emit("sync://upload-progress", mid_progress.clone());

        {
            let backend = cloud.lock().await;
            if let Err(err) = backend.notify_upload_complete(payload.clone()).await {
                return Err(emit_error(
                    UploadErrorPayload {
                        version_id: payload.version_id.clone(),
                        stage: "notify".to_string(),
                        reason: "backend_error".to_string(),
                        message: err.to_string(),
                        status: None,
                    },
                    &self.app_handle,
                ));
            }
        }

        let _ = self.app_handle.emit(
            "sync://upload-progress",
            UploadProgressPayload {
                version_id: job.version_id.clone(),
                progress: 100,
            },
        );

        let _ = self.app_handle.emit(
            "sync://upload-complete",
            UploadCompletePayload {
                version_id: payload.version_id,
            },
        );

        Ok(())
    }
}

// ============================================================================
// Sync Engine
// ============================================================================

pub struct SyncManager {
    pub queue: Arc<UploadQueue>,
    pub cloud: Arc<Mutex<Box<dyn CloudBackend + Send>>>,
    pub history: Arc<HistoryManager>,
    pub profiles: Arc<RwLock<ProfileManager>>,
    pub settings: Arc<SettingsManager>,
    online: Arc<AtomicBool>,
    paused: Arc<AtomicBool>,
    connection_status: Arc<RwLock<ConnectionStatus>>,
    app_handle: AppHandle,
    sync_trigger: Arc<Notify>,
    running: Arc<AtomicBool>,
}

impl SyncManager {
    pub fn new(
        app_handle: AppHandle,
        cloud: Arc<Mutex<Box<dyn CloudBackend + Send>>>,
        history: Arc<HistoryManager>,
        profiles: Arc<RwLock<ProfileManager>>,
        settings: Arc<SettingsManager>,
    ) -> Self {
        let online = Arc::new(AtomicBool::new(true));
        let paused = Arc::new(AtomicBool::new(false));
        let queue = Arc::new(UploadQueue::new(app_handle.clone(), online.clone()));
        let connection_status = Arc::new(RwLock::new(ConnectionStatus {
            connected: false,
            last_success: None,
            last_error: None,
        }));

        Self {
            queue,
            cloud,
            history,
            profiles,
            settings,
            online,
            paused,
            connection_status,
            app_handle,
            sync_trigger: Arc::new(Notify::new()),
            running: Arc::new(AtomicBool::new(false)),
        }
    }

    pub fn trigger_sync(&self) {
        self.sync_trigger.notify_one();
    }

    pub fn start_background_task(&self) {
        info!("[SYNC] start_background_task() called - initializing background tasks");
        let queue = self.queue.clone();
        let cloud = self.cloud.clone();
        let cloud_for_monitor = self.cloud.clone(); // Clone for monitoring loop
        let settings_for_ping = self.settings.clone();
        let app_for_ping = self.app_handle.clone();
        let online_flag = self.online.clone();
        let queue_for_online = self.queue.clone();
        let trigger_for_online = self.sync_trigger.clone();

        let settings_for_queue = self.settings.clone();

        // Spawn Queue Processor
        tokio::spawn(async move {
            queue.process_queue(cloud, settings_for_queue).await;
        });

        // Enhanced connectivity monitor with connection status tracking
        let connection_status_clone = self.connection_status.clone();
        tokio::spawn(async move {
            info!("[SYNC] Connection monitoring loop started");
            loop {
                // Check connection using backend's check_connection method
                debug!("[SYNC] Checking connection...");
                let backend = cloud_for_monitor.lock().await;
                let connected = backend.check_connection().await.unwrap_or(false);
                drop(backend);
                
                info!("[SYNC] Connection status: {}", if connected { "ONLINE" } else { "OFFLINE" });
                
                let previous = online_flag.swap(connected, Ordering::SeqCst);
                
                // Update connection status
                if let Ok(mut status) = connection_status_clone.write() {
                    status.connected = connected;
                    if connected {
                        status.last_success = Some(
                            std::time::SystemTime::now()
                                .duration_since(std::time::UNIX_EPOCH)
                                .unwrap_or_default()
                                .as_secs()
                        );
                        status.last_error = None;
                    }
                    
                    // Emit connection status event to frontend
                    let _ = app_for_ping.emit("connection-status", status.clone());
                    debug!("[SYNC] Emitted connection-status event: connected={}", status.connected);
                }

                // Handle state transitions
                if connected && !previous {
                    info!("[SYNC] Connection restored - triggering sync");
                    let _ = app_for_ping.emit("sync://online", "online");
                    queue_for_online.signal_online();
                    trigger_for_online.notify_one();
                } else if !connected && previous {
                    warn!("[SYNC] Connection lost");
                    let _ = app_for_ping.emit("sync://offline", "offline");
                }

                sleep(Duration::from_secs(30)).await; // Check every 30 seconds
            }
        });

        // Spawn Sync Loop
        let queue_clone = self.queue.clone();
        let cloud_clone = self.cloud.clone();
        let history_clone = self.history.clone();
        let profiles_clone = self.profiles.clone();
        let settings_clone = self.settings.clone();
        let app_handle_clone = self.app_handle.clone();
        let sync_trigger = self.sync_trigger.clone();
        let running_flag = self.running.clone();
        let online_status = self.online.clone();
        let paused_flag = self.paused.clone();

        tokio::spawn(async move {
            if running_flag.swap(true, Ordering::SeqCst) {
                return;
            }

            queue_clone.load_from_disk().await;
            loop {
                // Wait for either timeout (10s) or manual trigger
                tokio::select! {
                    _ = sleep(Duration::from_secs(10)) => {},
                    _ = sync_trigger.notified() => {
                        // We can't easily get tag here without settings, but we'll get it below
                    }
                }

                // Check if cloud mode is enabled (not Off)
                let mode = settings_clone
                    .get_settings()
                    .map(|s| s.cloud_mode)
                    .unwrap_or(crate::core::settings::CloudMode::Off);

                let tag = log_tag(&mode);

                // Skip sync if cloud mode is Off
                if mode == crate::core::settings::CloudMode::Off {
                    continue;
                }

                if paused_flag.load(Ordering::SeqCst) {
                    info!("{} [SYNC] Sync paused; skipping cycle", tag);
                    continue;
                }

                if !online_status.load(Ordering::SeqCst) {
                    info!("{} [SYNC] Skipping sync cycle while offline", tag);
                    continue;
                }

                info!("{} [SYNC] Starting sync cycle...", tag);

                // 1. List all games from History
                let games = history_clone.get_games();
                let current_device = settings_clone
                    .get_settings()
                    .map(|s| s.cloud.device_id)
                    .unwrap_or_default();

                for game_id in games {
                    // Get Local State
                    let local_latest = history_clone.get_latest_version(&game_id);

                    // Get Cloud State
                    let backend = cloud_clone.lock().await;
                    let cloud_versions = match backend.list_versions(game_id.clone(), Some(1)).await
                    {
                        Ok(versions) => versions,
                        Err(err) => {
                            warn!(
                                "{} [SYNC] Failed to list versions for {}: {}",
                                tag, game_id, err
                            );
                            let _ = app_handle_clone.emit(
                                "sync://cloud-list-error",
                                json!({ "gameId": game_id, "message": err.to_string() }),
                            );
                            continue;
                        }
                    };
                    drop(backend); // Release lock

                    // Decide
                    let decision = determine_sync_action(
                        local_latest.as_ref(),
                        &cloud_versions,
                        &current_device,
                    );

                    match decision {
                        SyncDecision::Upload => {
                            if let Some(local) = local_latest {
                                info!("{} [SYNC] Queueing upload for {}", tag, game_id);
                                queue_clone
                                    .add_job(UploadJob {
                                        game_id: game_id.clone(),
                                        emulator_id: local.metadata.emulator_id.clone(),
                                        version_id: local.metadata.version_id.clone(),
                                        archive_path: PathBuf::from(&local.archive_path),
                                        metadata: local.metadata.clone(),
                                        created_at: DateTime::from_timestamp(
                                            local.metadata.timestamp as i64,
                                            0,
                                        )
                                        .unwrap_or_default(),
                                        retries: 0,
                                        status: UploadStatus::Pending,
                                        total_size: fs::metadata(&local.archive_path)
                                            .map(|m| m.len())
                                            .unwrap_or_default(),
                                        hash: local.metadata.hash.clone(),
                                    })
                                    .await;
                            }
                        }
                        SyncDecision::Download(version_id) => {
                            info!(
                                "{} [SYNC] Should download {} version {}",
                                tag, game_id, version_id
                            );
                            if let Err(err) = perform_download(
                                cloud_clone.clone(),
                                history_clone.clone(),
                                profiles_clone.clone(),
                                app_handle_clone.clone(),
                                settings_clone.clone(),
                                game_id.clone(),
                                version_id,
                            )
                            .await
                            {
                                warn!("{} [SYNC] Download failed: {}", tag, err);
                            }
                        }
                        SyncDecision::Conflict => {
                            warn!("{} [SYNC] Conflict detected for {}", tag, game_id);
                            let _ = app_handle_clone.emit("sync://conflict-detected", game_id);
                        }
                        SyncDecision::Noop => {}
                    }
                }
            }
        });
    }
}

async fn check_online(settings: &Arc<SettingsManager>) -> bool {
    let config = match settings.get_settings() {
        Ok(s) => s.cloud,
        Err(err) => {
            warn!("[SYNC] Failed to read settings for ping: {}", err);
            return false;
        }
    };

    let base_url = config.base_url.trim_end_matches('/').to_string();
    if base_url.is_empty() {
        warn!("[SYNC] Cloud base_url missing; marking offline");
        return false;
    }

    let timeout = Duration::from_secs(config.timeout_seconds.max(1));
    let client = match Client::builder().timeout(timeout).build() {
        Ok(client) => client,
        Err(err) => {
            warn!("[SYNC] Failed to build ping client: {}", err);
            return false;
        }
    };

    match client.head(format!("{}/ping", base_url)).send().await {
        Ok(response) => response.status().is_success(),
        Err(err) => {
            debug!("[SYNC] Ping failed: {}", err);
            false
        }
    }
}

impl Clone for SyncManager {
    fn clone(&self) -> Self {
        Self {
            queue: self.queue.clone(),
            cloud: self.cloud.clone(),
            history: self.history.clone(),
            profiles: self.profiles.clone(),
            settings: self.settings.clone(),
            online: self.online.clone(),
            paused: self.paused.clone(),
            connection_status: self.connection_status.clone(),
            app_handle: self.app_handle.clone(),
            sync_trigger: self.sync_trigger.clone(),
            running: self.running.clone(),
        }
    }
}

impl SyncManager {
    pub fn pause(&self) {
        self.paused.store(true, Ordering::SeqCst);
    }

    pub fn resume(&self) {
        let was_paused = self.paused.swap(false, Ordering::SeqCst);
        if was_paused {
            self.sync_trigger.notify_one();
        }
    }

    pub fn is_paused(&self) -> bool {
        self.paused.load(Ordering::SeqCst)
    }
}

pub async fn perform_download(
    cloud: Arc<Mutex<Box<dyn CloudBackend + Send>>>,
    history: Arc<HistoryManager>,
    profiles: Arc<RwLock<ProfileManager>>,
    app_handle: AppHandle,
    settings: Arc<SettingsManager>,
    game_id: String,
    version_id: String,
) -> Result<(), String> {
    let downloads_dir = app_handle
        .path()
        .app_data_dir()
        .map_err(|e| format!("path error: {e}"))?
        .join("data")
        .join("cloud_downloads");
    if let Some(parent) = downloads_dir.parent() {
        let _ = fs::create_dir_all(parent);
    }
    fs::create_dir_all(&downloads_dir)
        .map_err(|e| format!("Failed to prepare downloads dir: {e}"))?;

    let target_path = downloads_dir.join(format!("{}_{}.zip", game_id, version_id));

    let emit_error = |stage: &str, message: String, app: &AppHandle| {
        let _ = app.emit(
            "sync://download-error",
            DownloadErrorPayload {
                version_id: version_id.clone(),
                stage: stage.to_string(),
                message: message.clone(),
            },
        );
        message
    };

    let download_info: DownloadUrlResponse = {
        ensure_registered_device_for_sync(&cloud, &settings, &app_handle)
            .await
            .map_err(|e| emit_error("request-url", e, &app_handle))?;

        let backend = cloud.lock().await;
        backend
            .request_download_url(game_id.clone(), version_id.clone())
            .await
            .map_err(|e| emit_error("request-url", e.to_string(), &app_handle))?
    };

    let total_bytes = download_info.size_bytes;
    let mut received_bytes: u64 = 0;
    let _ = app_handle.emit(
        "sync://download-progress",
        DownloadProgressPayload {
            version_id: version_id.clone(),
            received_bytes,
            total_bytes,
        },
    );

    let client = Client::new();
    let response = client
        .get(&download_info.download_url)
        .send()
        .await
        .map_err(|e| emit_error("http-get", e.to_string(), &app_handle))?;

    if !response.status().is_success() {
        return Err(emit_error(
            "http-get",
            format!("download failed: {}", response.status()),
            &app_handle,
        ));
    }

    let mut file = tokio::fs::File::create(&target_path)
        .await
        .map_err(|e| emit_error("write-file", e.to_string(), &app_handle))?;

    // Download the entire file
    let bytes = response
        .bytes()
        .await
        .map_err(|e| emit_error("http-get", e.to_string(), &app_handle))?;
    
    received_bytes = bytes.len() as u64;
    file.write_all(&bytes)
        .await
        .map_err(|e| emit_error("write-file", e.to_string(), &app_handle))?;

    let _ = app_handle.emit(
        "sync://download-progress",
        DownloadProgressPayload {
            version_id: version_id.clone(),
            received_bytes,
            total_bytes,
        },
    );

    file.flush()
        .await
        .map_err(|e| emit_error("write-file", e.to_string(), &app_handle))?;

    let emulator_id = download_info.emulator_id.clone().unwrap_or_default();
    if emulator_id.trim().is_empty() {
        return Err(emit_error(
            "unzip",
            "emulator_id missing in download metadata".to_string(),
            &app_handle,
        ));
    }

    let profile = {
        let guard = profiles
            .read()
            .map_err(|e| emit_error("unzip", format!("profile lock error: {e}"), &app_handle))?;
        guard
            .get_profile(&emulator_id)
            .map_err(|e| emit_error("unzip", e.to_string(), &app_handle))?
    };

    let Some(profile) = profile else {
        return Err(emit_error(
            "unzip",
            format!("profile not found for {emulator_id}"),
            &app_handle,
        ));
    };

    let Some(save_path) = profile.default_save_paths.first() else {
        return Err(emit_error(
            "unzip",
            format!("no default save path for {emulator_id}"),
            &app_handle,
        ));
    };

    let target_dir = PathBuf::from(save_path);
    if !target_dir.exists() {
        return Err(emit_error(
            "unzip",
            format!("save directory missing: {}", target_dir.display()),
            &app_handle,
        ));
    }

    let file = fs::File::open(&target_path)
        .map_err(|e| emit_error("unzip", e.to_string(), &app_handle))?;
    let mut archive =
        ZipArchive::new(file).map_err(|e| emit_error("unzip", e.to_string(), &app_handle))?;

    for i in 0..archive.len() {
        let mut file = archive
            .by_index(i)
            .map_err(|e| emit_error("unzip", e.to_string(), &app_handle))?;
        let Some(name) = file.enclosed_name().map(|p| p.to_owned()) else {
            continue;
        };

        let out_path = target_dir.join(name);
        if file.is_dir() {
            fs::create_dir_all(&out_path)
                .map_err(|e| emit_error("unzip", e.to_string(), &app_handle))?;
        } else {
            if let Some(parent) = out_path.parent() {
                fs::create_dir_all(parent)
                    .map_err(|e| emit_error("unzip", e.to_string(), &app_handle))?;
            }
            let mut outfile = fs::File::create(&out_path)
                .map_err(|e| emit_error("unzip", e.to_string(), &app_handle))?;
            std::io::copy(&mut file, &mut outfile)
                .map_err(|e| emit_error("unzip", e.to_string(), &app_handle))?;
        }
    }

    let timestamp = download_info
        .timestamp
        .unwrap_or_else(|| Utc::now().timestamp().max(0) as u64);
    let metadata = SaveMetadata {
        game_id: download_info.game_id.clone(),
        emulator_id: emulator_id.clone(),
        timestamp,
        version_id: download_info.version_id.clone(),
        file_list: download_info.file_list.clone(),
        hash: download_info.sha256.clone(),
        size_bytes: Some(download_info.size_bytes),
        sha256: Some(download_info.sha256.clone()),
        source: Some("cloud".to_string()),
    };

    history
        .add_version_from_cloud(metadata, target_path.clone())
        .map_err(|e| emit_error("write-history", e.to_string(), &app_handle))?;

    let _ = app_handle.emit(
        "sync://download-complete",
        DownloadCompletePayload {
            version_id: version_id.clone(),
        },
    );

    Ok(())
}

fn calculate_sha256(path: &PathBuf) -> Result<String, std::io::Error> {
    let mut file = fs::File::open(path)?;
    let mut hasher = Sha256::new();
    std::io::copy(&mut file, &mut hasher)?;
    let result = hasher.finalize();
    Ok(format!("{:x}", result))
}
