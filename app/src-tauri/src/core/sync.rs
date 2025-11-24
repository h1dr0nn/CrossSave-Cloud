use std::collections::VecDeque;
use std::fs;
use std::path::PathBuf;
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};
use std::time::Duration;

use chrono::{DateTime, Utc};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use tauri::{AppHandle, Emitter, Manager};
use tokio::sync::{Mutex, Notify};
use tokio::time::sleep;
use tracing::{debug, error, info, warn};

use crate::core::cloud::{log_tag, CloudBackend, CloudVersionSummary};
use crate::core::history::{HistoryEntry, HistoryManager};
use crate::core::packager::SaveMetadata;
use crate::core::settings::SettingsManager;
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

/// Determines the sync action based on local and cloud state.
///
/// Rules:
/// - If cloud has no versions -> Upload local (if exists)
/// - If local has no history -> Download latest cloud
/// - If both exist:
///   - Compare timestamps.
///   - If difference < 2s AND hashes differ -> Conflict
///   - If local > cloud -> Upload
///   - If cloud > local -> Download
///   - If equal -> Noop
pub fn determine_sync_action(
    local_entry: Option<&HistoryEntry>,
    cloud_versions: &[CloudVersionSummary],
    current_device: &str,
) -> SyncDecision {
    let latest_cloud = cloud_versions.iter().max_by_key(|v| v.timestamp);

    match (local_entry, latest_cloud) {
        (Some(_), None) => SyncDecision::Upload,
        (None, Some(cloud)) => SyncDecision::Download(cloud.version_id.clone()),
        (None, None) => SyncDecision::Noop,
        (Some(local), Some(cloud)) => {
            let local_time = local.metadata.timestamp;
            let cloud_time = cloud.timestamp;

            if cloud.hash == local.metadata.hash {
                return SyncDecision::Noop;
            }

            let time_diff = if cloud_time > local_time {
                cloud_time - local_time
            } else {
                local_time - cloud_time
            };

            let conflict_window = time_diff <= 2;
            let different_device = !cloud.device_id.is_empty() && cloud.device_id != current_device;

            if conflict_window && different_device {
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
        let mut q = self.queue.lock().await;
        // Avoid duplicates
        if !q.iter().any(|j| j.version_id == job.version_id) {
            q.push_back(job);
            self.notify.notify_one();
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
                for mut job in jobs {
                    if job.status == UploadStatus::Uploading {
                        job.status = UploadStatus::Pending;
                    }
                    q.push_back(job);
                }
            }
        }
    }

    pub async fn save_to_disk(&self) {
        if let Some(parent) = self.queue_path.parent() {
            let _ = fs::create_dir_all(parent);
        }
        let q = self.queue.lock().await;
        if let Ok(json) = serde_json::to_string_pretty(&q.clone().into_iter().collect::<Vec<_>>()) {
            let _ = fs::write(&self.queue_path, json);
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
        loop {
            if !self.online_status.load(Ordering::SeqCst) {
                self.wait_for_online().await;
                continue;
            }

            // Wait for a job
            let job = {
                let mut q = self.queue.lock().await;
                if q.is_empty() {
                    None
                } else {
                    q.pop_front()
                }
            };

            if let Some(mut job) = job {
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
                let result = self.perform_upload(&job, &cloud).await;

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
                // Wait for notification
                tokio::select! {
                    _ = self.notify.notified() => {},
                    _ = self.online_notify.notified() => {},
                }
            }
        }
    }

    async fn perform_upload(
        &self,
        job: &UploadJob,
        cloud: &Arc<Mutex<Box<dyn CloudBackend + Send>>>,
    ) -> Result<(), String> {
        let backend = cloud.lock().await;
        let mut metadata = job.metadata.clone();
        let hash = calculate_sha256(&job.archive_path).map_err(|e| e.to_string())?;
        metadata.hash = hash;

        backend
            .upload_archive(metadata, job.archive_path.clone())
            .await
            .map(|_| ())
            .map_err(|e| e.to_string())
    }
}

// ============================================================================
// Sync Engine
// ============================================================================

pub struct SyncManager {
    pub queue: Arc<UploadQueue>,
    pub cloud: Arc<Mutex<Box<dyn CloudBackend + Send>>>,
    pub history: Arc<HistoryManager>,
    pub settings: Arc<SettingsManager>,
    online: Arc<AtomicBool>,
    paused: Arc<AtomicBool>,
    app_handle: AppHandle,
    sync_trigger: Arc<Notify>,
    running: Arc<AtomicBool>,
}

impl SyncManager {
    pub fn new(
        app_handle: AppHandle,
        cloud: Arc<Mutex<Box<dyn CloudBackend + Send>>>,
        history: Arc<HistoryManager>,
        settings: Arc<SettingsManager>,
    ) -> Self {
        let online = Arc::new(AtomicBool::new(true));
        let paused = Arc::new(AtomicBool::new(false));
        let queue = Arc::new(UploadQueue::new(app_handle.clone(), online.clone()));

        Self {
            queue,
            cloud,
            history,
            settings,
            online,
            paused,
            app_handle,
            sync_trigger: Arc::new(Notify::new()),
            running: Arc::new(AtomicBool::new(false)),
        }
    }

    pub fn trigger_sync(&self) {
        self.sync_trigger.notify_one();
    }

    pub fn start_background_task(&self) {
        let queue = self.queue.clone();
        let cloud = self.cloud.clone();
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

        // Connectivity monitor
        tokio::spawn(async move {
            loop {
                let online = check_online(&settings_for_ping).await;
                let previous = online_flag.swap(online, Ordering::SeqCst);

                if online && !previous {
                    let _ = app_for_ping.emit("sync://online", "online");
                    queue_for_online.signal_online();
                    trigger_for_online.notify_one();
                } else if !online && previous {
                    let _ = app_for_ping.emit("sync://offline", "offline");
                }

                sleep(Duration::from_secs(15)).await;
            }
        });

        // Spawn Sync Loop
        let queue_clone = self.queue.clone();
        let cloud_clone = self.cloud.clone();
        let history_clone = self.history.clone();
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

                // Check if sync is enabled
                let (enabled, mode) = settings_clone
                    .get_settings()
                    .map(|s| (s.cloud.enabled, s.cloud_mode))
                    .unwrap_or((false, crate::core::settings::CloudMode::Off));
                
                let tag = log_tag(&mode);

                if !enabled {
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
                    let cloud_versions = backend
                        .list_versions(game_id.clone(), Some(1))
                        .await
                        .unwrap_or_default();
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
                            info!("{} [SYNC] Should download {} version {}", tag, game_id, version_id);
                            if let Err(err) = perform_download(
                                cloud_clone.clone(),
                                history_clone.clone(),
                                app_handle_clone.clone(),
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
            settings: self.settings.clone(),
            online: self.online.clone(),
            paused: self.paused.clone(),
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

async fn perform_download(
    cloud: Arc<Mutex<Box<dyn CloudBackend + Send>>>,
    history: Arc<HistoryManager>,
    app_handle: AppHandle,
    game_id: String,
    version_id: String,
) -> Result<(), String> {
    let downloads_dir = app_handle
        .path()
        .app_data_dir()
        .map_err(|e| format!("path error: {e}"))?
        .join("data")
        .join("cloud_downloads");
    let target_path = downloads_dir.join(format!("{}_{}.zip", game_id, version_id));

    let start_progress = DownloadProgressPayload {
        version_id: version_id.clone(),
        progress: 0,
    };
    let _ = app_handle.emit("sync://download-progress", start_progress);

    let result: Result<(), String> = async {
        let summary = {
            let backend = cloud.lock().await;
            let versions = backend
                .list_versions(game_id.clone(), None)
                .await
                .map_err(|e| e.to_string())?;
            versions
                .into_iter()
                .find(|v| v.version_id == version_id)
                .ok_or_else(|| "version not found in cloud".to_string())?
        };

        {
            let backend = cloud.lock().await;
            backend
                .download_version(game_id.clone(), version_id.clone(), target_path.clone())
                .await
                .map_err(|e| e.to_string())?;
        }

        let mid_progress = DownloadProgressPayload {
            version_id: version_id.clone(),
            progress: 50,
        };
        let _ = app_handle.emit("sync://download-progress", mid_progress);

        // Extract archive
        if let Some(parent) = downloads_dir.parent() {
            let _ = fs::create_dir_all(parent);
        }

        let extract_dir = history.base_dir.join(&game_id).join("downloaded");
        let file = fs::File::open(&target_path).map_err(|e| e.to_string())?;
        let mut archive = ZipArchive::new(file).map_err(|e| e.to_string())?;
        if extract_dir.exists() {
            let _ = fs::remove_dir_all(&extract_dir);
        }
        fs::create_dir_all(&extract_dir).map_err(|e| e.to_string())?;
        for i in 0..archive.len() {
            let mut file = archive.by_index(i).map_err(|e| e.to_string())?;
            let out_path = extract_dir.join(file.name());
            if file.is_dir() {
                fs::create_dir_all(&out_path).map_err(|e| e.to_string())?;
            } else {
                if let Some(parent) = out_path.parent() {
                    fs::create_dir_all(parent).map_err(|e| e.to_string())?;
                }
                let mut outfile = fs::File::create(&out_path).map_err(|e| e.to_string())?;
                std::io::copy(&mut file, &mut outfile).map_err(|e| e.to_string())?;
            }
        }

        let metadata = SaveMetadata {
            game_id: summary.game_id.clone(),
            emulator_id: summary.emulator_id.clone(),
            timestamp: summary.timestamp,
            version_id: summary.version_id.clone(),
            file_list: summary.file_list.clone(),
            hash: summary.hash.clone(),
        };

        history
            .save_to_history(metadata, target_path.clone())
            .map_err(|e| e.to_string())?;

        Ok(())
    }
    .await;

    match result {
        Ok(_) => {
            let _ = app_handle.emit(
                "sync://download-progress",
                DownloadProgressPayload {
                    version_id: version_id.clone(),
                    progress: 100,
                },
            );
            let _ = app_handle.emit(
                "sync://download-complete",
                DownloadCompletePayload {
                    version_id: version_id.clone(),
                    path: target_path.to_string_lossy().to_string(),
                },
            );
            Ok(())
        }
        Err(err) => {
            let _ = app_handle.emit(
                "sync://download-error",
                DownloadErrorPayload {
                    version_id: version_id.clone(),
                    message: err.clone(),
                },
            );
            Err(err)
        }
    }
}

fn calculate_sha256(path: &PathBuf) -> Result<String, std::io::Error> {
    let mut file = fs::File::open(path)?;
    let mut hasher = Sha256::new();
    std::io::copy(&mut file, &mut hasher)?;
    let result = hasher.finalize();
    Ok(format!("{:x}", result))
}
