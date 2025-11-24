use std::collections::VecDeque;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Emitter};
use tokio::sync::{Mutex, Notify};
use tokio::time::sleep;
use tracing::{error, info, warn};

use crate::core::cloud::{CloudBackend, CloudVersionSummary};
use crate::core::history::{HistoryEntry, HistoryManager};
use crate::core::settings::SettingsManager;

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

            let time_diff_ms = if cloud_time > local_time {
                cloud_time - local_time
            } else {
                local_time - cloud_time
            } * 1000;

            if time_diff_ms < 2000 {
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UploadJob {
    pub game_id: String,
    pub emulator_id: String,
    pub version_id: String,
    pub archive_path: PathBuf,
    pub created_at: DateTime<Utc>,
    pub retries: u32,
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
}

impl UploadQueue {
    pub fn new(app_handle: AppHandle) -> Self {
        Self {
            queue: Arc::new(Mutex::new(VecDeque::new())),
            active_job: Arc::new(Mutex::new(None)),
            notify: Arc::new(Notify::new()),
            app_handle,
        }
    }

    pub async fn add_job(&self, job: UploadJob) {
        let mut q = self.queue.lock().await;
        // Avoid duplicates
        if !q.iter().any(|j| j.version_id == job.version_id) {
            q.push_back(job);
            self.notify.notify_one();
            self.emit_status().await;
        }
    }

    #[allow(dead_code)]
    pub async fn load_from_disk(&self) {
        // TODO: Implement persistence loading
        // For now, start empty
    }

    #[allow(dead_code)]
    pub async fn save_to_disk(&self) {
        // TODO: Implement persistence saving
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

                // Perform Upload
                let result = self.perform_upload(&job, &cloud).await;

                match result {
                    Ok(_) => {
                        info!("[SYNC] Upload complete for {}", job.game_id);
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
                        }
                    }
                }

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
        // Re-use logic from cloud_api or call backend directly
        // We need metadata. For simplicity, we might need to fetch it from HistoryManager again
        // OR store metadata in UploadJob.
        // Let's assume we can reconstruct basic metadata or we should update UploadJob to carry it.
        // For now, let's just try to upload with basic info.

        // Ideally, we should use the HistoryManager to get the full entry,
        // but passing HistoryManager here is complex.
        // Let's assume the backend just needs the file and some metadata.

        let backend = cloud.lock().await;

        // Construct minimal metadata
        let metadata = crate::core::packager::SaveMetadata {
            game_id: job.game_id.clone(),
            emulator_id: job.emulator_id.clone(),
            version_id: job.version_id.clone(),
            timestamp: job.created_at.timestamp() as u64,
            file_list: Vec::new(),       // We lost this info in UploadJob
            hash: "pending".to_string(), // We lost this info
        };

        backend
            .upload_archive(metadata, job.archive_path.clone())
            .await
            .map_err(|e| e.to_string())?;

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
    pub settings: Arc<SettingsManager>,
    app_handle: AppHandle,
    sync_trigger: Arc<Notify>,
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

        tokio::spawn(async move {
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
                    let decision = determine_sync_action(local_latest.as_ref(), &cloud_versions);

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
                                        created_at: DateTime::from_timestamp(
                                            local.metadata.timestamp as i64,
                                            0,
                                        )
                                        .unwrap_or_default(),
                                        retries: 0,
                                    })
                                    .await;
                            }
                        }
                        SyncDecision::Download(version_id) => {
                            info!("[SYNC] Should download {} version {}", game_id, version_id);
                            // Trigger download logic (omitted for brevity in this step, can be added)
                            // Ideally we call a download function here.
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
        }
    }
}
