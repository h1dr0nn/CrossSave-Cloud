use std::{
    collections::HashMap,
    fs,
    path::{Path, PathBuf},
    sync::Mutex,
};

use serde::{Deserialize, Serialize};
use thiserror::Error;
use tracing::{error, info, warn};

use crate::core::packager::{PackagedSave, SaveMetadata};

const DEFAULT_RETENTION: usize = 10;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct HistoryEntry {
    pub archive_path: String,
    pub metadata_path: String,
    pub metadata: SaveMetadata,
}

#[derive(Debug, Error)]
pub enum HistoryError {
    #[error("io error: {0}")]
    Io(String),
    #[error("serialization error: {0}")]
    Serialization(String),
    #[error("invalid input: {0}")]
    InvalidInput(String),
    #[error("cache lock error: {0}")]
    Lock(String),
    #[error("history item not found: {0}")]
    NotFound(String),
}

#[derive(Debug)]
pub struct HistoryManager {
    pub base_dir: PathBuf,
    cache: Mutex<HashMap<String, Vec<HistoryEntry>>>,
    retention_limit: Mutex<usize>,
    auto_delete: Mutex<bool>,
}

impl HistoryManager {
    pub fn init(base_dir: PathBuf, retention_limit: usize, auto_delete: bool) -> Result<Self, HistoryError> {
        fs::create_dir_all(&base_dir).map_err(|err| HistoryError::Io(err.to_string()))?;

        let mut cache: HashMap<String, Vec<HistoryEntry>> = HashMap::new();

        let entries = fs::read_dir(&base_dir).map_err(|err| HistoryError::Io(err.to_string()))?;

        for entry in entries {
            let entry = match entry {
                Ok(value) => value,
                Err(err) => {
                    warn!("[HISTORY] Failed to read game directory: {err}");
                    continue;
                }
            };

            let path = entry.path();
            if !path.is_dir() {
                continue;
            }

            let Some(game_id) = path
                .file_name()
                .and_then(|name| name.to_str())
                .map(String::from)
            else {
                warn!("[HISTORY] Skipping invalid game directory name: {:?}", path);
                continue;
            };

            let mut history_entries = Self::load_history_entries(&path, &game_id)?;
            if auto_delete {
                Self::enforce_retention(&mut history_entries, retention_limit)?;
            }

            cache.insert(game_id, history_entries);
        }

        Ok(Self {
            base_dir,
            cache: Mutex::new(cache),
            retention_limit: Mutex::new(retention_limit),
            auto_delete: Mutex::new(auto_delete),
        })
    }

    pub fn with_defaults() -> Self {
        let base_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("data")
            .join("archives")
            .join("history");

        match Self::init(base_dir.clone(), DEFAULT_RETENTION, true) {
            Ok(manager) => manager,
            Err(err) => {
                error!("[HISTORY] Failed to initialize history manager: {err}");
                Self {
                    base_dir,
                    cache: Mutex::new(HashMap::new()),
                    retention_limit: Mutex::new(DEFAULT_RETENTION),
                    auto_delete: Mutex::new(true),
                }
            }
        }
    }

    pub fn set_policy(&self, retention_limit: usize, auto_delete: bool) -> Result<(), HistoryError> {
        {
            let mut retention_guard = self
                .retention_limit
                .lock()
                .map_err(|err| HistoryError::Lock(err.to_string()))?;
            *retention_guard = retention_limit;
        }

        {
            let mut auto_delete_guard = self
                .auto_delete
                .lock()
                .map_err(|err| HistoryError::Lock(err.to_string()))?;
            *auto_delete_guard = auto_delete;
        }

        if auto_delete {
            self.trim_all()?;
        }

        Ok(())
    }

    pub fn policy(&self) -> Result<(usize, bool), HistoryError> {
        let limit = *self
            .retention_limit
            .lock()
            .map_err(|err| HistoryError::Lock(err.to_string()))?;
        let auto_delete = *self
            .auto_delete
            .lock()
            .map_err(|err| HistoryError::Lock(err.to_string()))?;
        Ok((limit, auto_delete))
    }

    pub fn save_to_history(
        &self,
        metadata: SaveMetadata,
        archive_path: PathBuf,
    ) -> Result<HistoryEntry, HistoryError> {
        if metadata.game_id.trim().is_empty() {
            return Err(HistoryError::InvalidInput("game_id missing".into()));
        }

        if metadata.version_id.trim().is_empty() {
            return Err(HistoryError::InvalidInput("version_id missing".into()));
        }

        if !archive_path.exists() {
            return Err(HistoryError::InvalidInput(
                "archive path does not exist".into(),
            ));
        }

        let game_dir = self.base_dir.join(&metadata.game_id);
        fs::create_dir_all(&game_dir).map_err(|err| HistoryError::Io(err.to_string()))?;

        let archive_destination = game_dir.join(format!("{}.zip", metadata.version_id));
        fs::copy(&archive_path, &archive_destination)
            .map_err(|err| HistoryError::Io(err.to_string()))?;

        let metadata_destination = game_dir.join(format!("{}.json", metadata.version_id));
        let metadata_json = serde_json::to_string_pretty(&metadata)
            .map_err(|err| HistoryError::Serialization(err.to_string()))?;
        fs::write(&metadata_destination, metadata_json)
            .map_err(|err| HistoryError::Io(err.to_string()))?;

        let entry = HistoryEntry {
            archive_path: archive_destination.to_string_lossy().to_string(),
            metadata_path: metadata_destination.to_string_lossy().to_string(),
            metadata,
        };

        self.insert_entry(entry.clone())?;
        self.trim_cache_for_game(&entry.metadata.game_id)?;
        info!(
            "[HISTORY] Saved version {} to history",
            entry.metadata.version_id
        );

        Ok(entry)
    }

    pub fn list_history(&self, game_id: String) -> Result<Vec<HistoryEntry>, HistoryError> {
        let mut guard = self
            .cache
            .lock()
            .map_err(|err| HistoryError::Lock(err.to_string()))?;

        if let Some(entries) = guard.get_mut(&game_id) {
            entries.sort_by(|a, b| b.metadata.timestamp.cmp(&a.metadata.timestamp));
            return Ok(entries.clone());
        }

        let game_dir = self.base_dir.join(&game_id);
        if game_dir.exists() {
            let mut history_entries = Self::load_history_entries(&game_dir, &game_id)?;
            let (limit, auto_delete) = self.policy()?;
            if auto_delete {
                Self::enforce_retention(&mut history_entries, limit)?;
            }
            guard.insert(game_id.clone(), history_entries.clone());
            return Ok(history_entries);
        }

        Ok(Vec::new())
    }

    pub fn get_history_item(
        &self,
        game_id: String,
        version_id: String,
    ) -> Result<HistoryEntry, HistoryError> {
        let guard = self
            .cache
            .lock()
            .map_err(|err| HistoryError::Lock(err.to_string()))?;

        let Some(entries) = guard.get(&game_id) else {
            return Err(HistoryError::NotFound(format!("{game_id}:{version_id}")));
        };

        entries
            .iter()
            .find(|entry| entry.metadata.version_id == version_id)
            .cloned()
            .ok_or_else(|| HistoryError::NotFound(format!("{game_id}:{version_id}")))
    }

    pub fn rollback_version(
        &self,
        game_id: String,
        version_id: String,
    ) -> Result<PackagedSave, HistoryError> {
        let entry = self.get_history_item(game_id.clone(), version_id.clone())?;
        let archives_root = self.archives_root();
        fs::create_dir_all(&archives_root).map_err(|err| HistoryError::Io(err.to_string()))?;

        let archive_destination = archives_root.join(format!("{}_{}.zip", game_id, version_id));
        fs::copy(Path::new(&entry.archive_path), &archive_destination)
            .map_err(|err| HistoryError::Io(err.to_string()))?;

        let metadata_destination = archives_root.join(format!("{}_{}.json", game_id, version_id));
        let metadata_json = serde_json::to_string_pretty(&entry.metadata)
            .map_err(|err| HistoryError::Serialization(err.to_string()))?;
        fs::write(&metadata_destination, metadata_json)
            .map_err(|err| HistoryError::Io(err.to_string()))?;

        info!(
            "[HISTORY] Rolled back {game_id} to version {}",
            entry.metadata.version_id
        );

        Ok(PackagedSave {
            archive_path: archive_destination.to_string_lossy().to_string(),
            metadata: entry.metadata,
        })
    }

    pub fn delete_history_item(
        &self,
        game_id: String,
        version_id: String,
    ) -> Result<(), HistoryError> {
        let mut guard = self
            .cache
            .lock()
            .map_err(|err| HistoryError::Lock(err.to_string()))?;

        let Some(entries) = guard.get_mut(&game_id) else {
            return Err(HistoryError::NotFound(format!("{game_id}:{version_id}")));
        };

        if let Some(index) = entries
            .iter()
            .position(|entry| entry.metadata.version_id == version_id)
        {
            let removed = entries.remove(index);
            Self::remove_files(&removed)?;
            info!("[HISTORY] Deleted version {version_id} for {game_id}");
            return Ok(());
        }

        Err(HistoryError::NotFound(format!("{game_id}:{version_id}")))
    }

    pub fn clear_all(&self) -> Result<(), HistoryError> {
        if self.base_dir.exists() {
            fs::remove_dir_all(&self.base_dir).map_err(|err| HistoryError::Io(err.to_string()))?;
        }

        fs::create_dir_all(&self.base_dir).map_err(|err| HistoryError::Io(err.to_string()))?;
        let mut guard = self
            .cache
            .lock()
            .map_err(|err| HistoryError::Lock(err.to_string()))?;
        guard.clear();

        info!("[HISTORY] Cleared history cache");
        Ok(())
    }

    pub fn total_size(&self) -> Result<u64, HistoryError> {
        calculate_dir_size(&self.base_dir)
    }

    fn load_history_entries(game_dir: &Path, game_id: &str) -> Result<Vec<HistoryEntry>, HistoryError> {
        let mut history_entries: Vec<HistoryEntry> = Vec::new();
        let files = match fs::read_dir(game_dir) {
            Ok(files) => files,
            Err(err) => {
                warn!("[HISTORY] Failed to read history folder for {game_id}: {err}");
                return Ok(history_entries);
            }
        };

        for file in files {
            let file = match file {
                Ok(value) => value,
                Err(err) => {
                    warn!("[HISTORY] Failed to read history file entry: {err}");
                    continue;
                }
            };

            let file_path = file.path();
            if file_path.extension().and_then(|ext| ext.to_str()) != Some("json") {
                continue;
            }

            match Self::load_entry(&file_path, game_id) {
                Ok(entry) => history_entries.push(entry),
                Err(err) => warn!("[HISTORY] Skipping malformed history entry: {err}"),
            }
        }

        history_entries.sort_by(|a, b| b.metadata.timestamp.cmp(&a.metadata.timestamp));
        Ok(history_entries)
    }

    fn load_entry(path: &Path, game_id: &str) -> Result<HistoryEntry, HistoryError> {
        let metadata_content =
            fs::read_to_string(path).map_err(|err| HistoryError::Io(err.to_string()))?;
        let metadata: SaveMetadata = serde_json::from_str(&metadata_content)
            .map_err(|err| HistoryError::Serialization(err.to_string()))?;

        if metadata.game_id != game_id {
            warn!(
                "[HISTORY] Metadata game_id mismatch for {:?}: expected {}, got {}",
                path, game_id, metadata.game_id
            );
        }

        let archive_path = path.with_extension("zip");
        if !archive_path.exists() {
            return Err(HistoryError::NotFound(format!(
                "archive missing for metadata {:?}",
                path
            )));
        }

        Ok(HistoryEntry {
            archive_path: archive_path.to_string_lossy().to_string(),
            metadata_path: path.to_string_lossy().to_string(),
            metadata,
        })
    }

    fn insert_entry(&self, entry: HistoryEntry) -> Result<(), HistoryError> {
        let mut guard = self
            .cache
            .lock()
            .map_err(|err| HistoryError::Lock(err.to_string()))?;
        let history = guard.entry(entry.metadata.game_id.clone()).or_default();
        history.retain(|existing| existing.metadata.version_id != entry.metadata.version_id);
        history.push(entry);
        history.sort_by(|a, b| b.metadata.timestamp.cmp(&a.metadata.timestamp));
        Ok(())
    }

    fn trim_cache_for_game(&self, game_id: &str) -> Result<(), HistoryError> {
        let (limit, auto_delete) = self.policy()?;
        if !auto_delete {
            return Ok(());
        }

        let mut guard = self
            .cache
            .lock()
            .map_err(|err| HistoryError::Lock(err.to_string()))?;
        if let Some(entries) = guard.get_mut(game_id) {
            Self::enforce_retention(entries, limit)?;
        }

        Ok(())
    }

    fn trim_all(&self) -> Result<(), HistoryError> {
        let (limit, auto_delete) = self.policy()?;
        if !auto_delete {
            return Ok(());
        }

        let mut guard = self
            .cache
            .lock()
            .map_err(|err| HistoryError::Lock(err.to_string()))?;

        for entries in guard.values_mut() {
            Self::enforce_retention(entries, limit)?;
        }

        Ok(())
    }

    fn enforce_retention(entries: &mut Vec<HistoryEntry>, limit: usize) -> Result<(), HistoryError> {
        while entries.len() > limit {
            if let Some(removed) = entries.pop() {
                warn!(
                    "[HISTORY] Removing oldest history entry {} for {}",
                    removed.metadata.version_id, removed.metadata.game_id
                );
                if let Err(err) = Self::remove_files(&removed) {
                    warn!("[HISTORY] Failed to remove trimmed entry: {err}");
                }
            } else {
                break;
            }
        }

        Ok(())
    }

    fn remove_files(entry: &HistoryEntry) -> Result<(), HistoryError> {
        if let Err(err) = fs::remove_file(&entry.archive_path) {
            warn!(
                "[HISTORY] Failed to delete archive {}: {}",
                &entry.archive_path, err
            );
        }

        if let Err(err) = fs::remove_file(&entry.metadata_path) {
            warn!(
                "[HISTORY] Failed to delete metadata {}: {}",
                &entry.metadata_path, err
            );
        }

        Ok(())
    }

    fn archives_root(&self) -> PathBuf {
        self.base_dir
            .parent()
            .map(Path::to_path_buf)
            .unwrap_or_else(|| self.base_dir.clone())
    }
}

fn calculate_dir_size(path: &Path) -> Result<u64, HistoryError> {
    if !path.exists() {
        return Ok(0);
    }

    let mut total = 0u64;
    for entry in fs::read_dir(path).map_err(|err| HistoryError::Io(err.to_string()))? {
        let entry = entry.map_err(|err| HistoryError::Io(err.to_string()))?;
        let metadata = entry
            .metadata()
            .map_err(|err| HistoryError::Io(err.to_string()))?;

        if metadata.is_file() {
            total += metadata.len();
        } else if metadata.is_dir() {
            total += calculate_dir_size(&entry.path())?;
        }
    }

    Ok(total)
}

// Example metadata JSON stored for each history entry
// {
//   "game_id": "super_mario_world",
//   "emulator_id": "retroarch",
//   "timestamp": 1700000000,
//   "version_id": "v1_abcdef",
//   "file_list": ["save.srm"],
//   "hash": "a1b2c3d4"
// }
