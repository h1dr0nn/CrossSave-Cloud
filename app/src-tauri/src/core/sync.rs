use std::collections::VecDeque;
use std::fs;
use std::path::PathBuf;
use std::sync::{atomic::{AtomicBool, Ordering}, Arc};
use std::time::Duration;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use tauri::{AppHandle, Emitter, Manager};
use tokio::sync::{Mutex, Notify};
use tokio::time::sleep;
use tracing::{error, info, warn};

use crate::core::cloud::{CloudBackend, CloudVersionSummary};
use crate::core::history::{HistoryEntry, HistoryManager};
use crate::core::settings::SettingsManager;
use crate::core::packager::SaveMetadata;
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
    app_handle: AppHandle,
    queue_path: PathBuf,
}

impl UploadQueue {
    pub fn new(app_handle: AppHandle) -> Self {
        let queue_path = app_handle
            .path()
            .app_data_dir()
            .map(|dir| dir.join("data").join("sync_queue.json"))
            .unwrap_or_else(|_| PathBuf::from("sync_queue.json"));
        Self {
            queue: Arc::new(Mutex::new(VecDeque::new())),
            active_job: Arc::new(Mutex::new(None)),
            notify: Arc::new(Notify::new()),
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

    async fn emit_status(&self) {
        let status = self.get_status().await;
        let _ = self.app_handle.emit("sync://status", status);
    }

    pub async fn process_queue(&self, cloud: Arc<Mutex<Box<dyn CloudBackend + Send>>>) {
        loop {
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

                info!("[SYNC] Starting upload for {}", job.game_id);

                job.status = UploadStatus::Uploading;
                self.save_to_disk().await;

                // Perform Upload
                let result = self.perform_upload(&job, &cloud).await;

                match result {
                    Ok(_) => {
                        info!("[SYNC] Upload complete for {}", job.game_id);
                        job.status = UploadStatus::Completed;
                    }
                    Err(e) => {
                        error!("[SYNC] Upload failed for {}: {}", job.game_id, e);
                        if job.retries < 3 {
                            job.retries += 1;
                            let retries = job.retries;
                            warn!("[SYNC] Retrying job (attempt {})", retries);
                            let mut q = self.queue.lock().await;
                            q.push_front(job); // Put back at front
                            sleep(Duration::from_secs(2u64.pow(retries))).await;
                        // Exponential backoff
                        } else {
                            error!("[SYNC] Job failed after 3 retries, dropping.");
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
                self.notify.notified().await;
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
        let queue = Arc::new(UploadQueue::new(app_handle.clone()));

        Self {
            queue,
            cloud,
            history,
            settings,
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

        // Spawn Queue Processor
        tokio::spawn(async move {
            queue.process_queue(cloud).await;
        });

        // Spawn Sync Loop
        let queue_clone = self.queue.clone();
        let cloud_clone = self.cloud.clone();
        let history_clone = self.history.clone();
        let settings_clone = self.settings.clone();
        let app_handle_clone = self.app_handle.clone();
        let sync_trigger = self.sync_trigger.clone();
        let running_flag = self.running.clone();

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
                        info!("[SYNC] Manual sync triggered");
                    }
                }

                // Check if sync is enabled
                let enabled = settings_clone
                    .get_settings()
                    .map(|s| s.cloud.enabled)
                    .unwrap_or(false);
                if !enabled {
                    continue;
                }

                info!("[SYNC] Starting sync cycle...");

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
                    let decision =
                        determine_sync_action(local_latest.as_ref(), &cloud_versions, &current_device);

                    match decision {
                        SyncDecision::Upload => {
                            if let Some(local) = local_latest {
                                info!("[SYNC] Queueing upload for {}", game_id);
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
                            info!("[SYNC] Should download {} version {}", game_id, version_id);
                            if let Err(err) = perform_download(
                                cloud_clone.clone(),
                                history_clone.clone(),
                                app_handle_clone.clone(),
                                game_id.clone(),
                                version_id,
                            )
                            .await
                            {
                                warn!("[SYNC] Download failed: {}", err);
                            }
                        }
                        SyncDecision::Conflict => {
                            warn!("[SYNC] Conflict detected for {}", game_id);
                            let _ = app_handle_clone.emit("sync://conflict-detected", game_id);
                        }
                        SyncDecision::Noop => {}
                    }
                }
            }
        });
    }
}

impl Clone for SyncManager {
    fn clone(&self) -> Self {
        Self {
            queue: self.queue.clone(),
            cloud: self.cloud.clone(),
            history: self.history.clone(),
            settings: self.settings.clone(),
            app_handle: self.app_handle.clone(),
            sync_trigger: self.sync_trigger.clone(),
            running: self.running.clone(),
        }
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

    let _ = app_handle.emit("sync://download-progress", 0u8);
    {
        let backend = cloud.lock().await;
        backend
            .download_version(game_id.clone(), version_id.clone(), target_path.clone())
            .await
            .map_err(|e| e.to_string())?;
    }

    let _ = app_handle.emit("sync://download-progress", 50u8);

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

    let _ = app_handle.emit("sync://download-progress", 100u8);
    let _ = app_handle.emit("sync://download-complete", version_id.clone());

    Ok(())
}

fn calculate_sha256(path: &PathBuf) -> Result<String, std::io::Error> {
    let mut file = fs::File::open(path)?;
    let mut hasher = Sha256::new();
    std::io::copy(&mut file, &mut hasher)?;
    let result = hasher.finalize();
    Ok(format!("{:x}", result))
}
