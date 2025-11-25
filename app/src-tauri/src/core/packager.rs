use std::{
    fs,
    io::{self, Read},
    path::{Path, PathBuf},
    time::{SystemTime, UNIX_EPOCH},
};

use glob::Pattern;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use thiserror::Error;
use tracing::{info, warn};
use zip::{write::FileOptions, CompressionMethod, ZipWriter};

#[derive(Debug, Error)]
pub enum PackagerError {
    #[error("no files found to package")]
    NoFiles,
    #[error("invalid input: {0}")]
    InvalidInput(String),
    #[error("io error: {0}")]
    Io(String),
    #[error("failed to create archive: {0}")]
    Archive(String),
    #[error("timestamp error: {0}")]
    Timestamp(String),
    #[error("missing archive for metadata generation")]
    MissingArchive,
    #[error("hash calculation failed: {0}")]
    Hash(String),
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SaveMetadata {
    pub game_id: String,
    pub emulator_id: String,
    pub timestamp: u64,
    pub version_id: String,
    pub file_list: Vec<String>,
    pub hash: String,
    #[serde(default)]
    pub size_bytes: Option<u64>,
    #[serde(default)]
    pub sha256: Option<String>,
    #[serde(default)]
    pub source: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PackagedSave {
    pub archive_path: String,
    pub metadata: SaveMetadata,
}

#[derive(Debug)]
pub struct SavePackager {
    game_id: String,
    emulator_id: String,
    archive_path: Option<PathBuf>,
    timestamp: Option<u64>,
    version_id: Option<String>,
}

impl SavePackager {
    pub fn new(game_id: String, emulator_id: String) -> Self {
        Self {
            game_id,
            emulator_id,
            archive_path: None,
            timestamp: None,
            version_id: None,
        }
    }

    pub fn collect_files(
        &self,
        paths: Vec<PathBuf>,
        patterns: Vec<String>,
    ) -> Result<Vec<PathBuf>, PackagerError> {
        let mut compiled_patterns: Vec<Pattern> = Vec::new();
        for pattern in patterns {
            if pattern.trim().is_empty() {
                continue;
            }

            match Pattern::new(&pattern) {
                Ok(p) => compiled_patterns.push(p),
                Err(err) => warn!("[PACKAGER] Ignoring invalid pattern {pattern}: {err}"),
            }
        }

        let mut files: Vec<PathBuf> = Vec::new();

        for path in paths {
            if path.as_os_str().is_empty() {
                continue;
            }

            match fs::metadata(&path) {
                Ok(metadata) => {
                    if metadata.is_dir() {
                        self.collect_from_directory(&path, &compiled_patterns, &mut files);
                    } else if metadata.is_file() {
                        if self.matches_patterns(&path, &compiled_patterns) {
                            files.push(path);
                        }
                    }
                }
                Err(err) => {
                    warn!("[PACKAGER] Skipping path {:?}: {}", path, err);
                }
            }
        }

        files.sort();
        files.dedup();

        if files.is_empty() {
            return Err(PackagerError::NoFiles);
        }

        info!("[PACKAGER] Found {} files for packaging", files.len());
        Ok(files)
    }

    pub fn create_archive(&mut self, files: Vec<PathBuf>) -> Result<PathBuf, PackagerError> {
        let version_id = self
            .version_id
            .clone()
            .ok_or_else(|| PackagerError::InvalidInput("version_id not set".into()))?;

        // Use system temp directory to avoid triggering file watcher
        let archives_dir = std::env::temp_dir().join("crosssave_archives");

        fs::create_dir_all(&archives_dir).map_err(|err| PackagerError::Io(err.to_string()))?;

        let archive_path = archives_dir.join(format!("{}_{}.zip", self.game_id, version_id));
        let file = fs::File::create(&archive_path)
            .map_err(|err| PackagerError::Archive(err.to_string()))?;
        let mut zip = ZipWriter::new(file);
        let options = FileOptions::default().compression_method(CompressionMethod::Deflated);

        for (index, file_path) in files.iter().enumerate() {
            let entry_name = self.entry_name(file_path, index);
            if entry_name.is_empty() {
                warn!("[PACKAGER] Skipping file with empty name: {:?}", file_path);
                continue;
            }

            zip.start_file(entry_name.clone(), options)
                .map_err(|err| PackagerError::Archive(err.to_string()))?;

            let mut source =
                fs::File::open(file_path).map_err(|err| PackagerError::Io(err.to_string()))?;
            io::copy(&mut source, &mut zip)
                .map_err(|err| PackagerError::Archive(err.to_string()))?;
        }

        zip.finish()
            .map_err(|err| PackagerError::Archive(err.to_string()))?;

        self.archive_path = Some(archive_path.clone());
        info!("[PACKAGER] Archive created at {:?}", archive_path);
        Ok(archive_path)
    }

    pub fn generate_metadata(&self, files: Vec<PathBuf>) -> Result<SaveMetadata, PackagerError> {
        let archive_path = self
            .archive_path
            .clone()
            .ok_or(PackagerError::MissingArchive)?;

        let timestamp = self
            .timestamp
            .ok_or_else(|| PackagerError::InvalidInput("timestamp not set".into()))?;

        let version_id = self
            .version_id
            .clone()
            .ok_or_else(|| PackagerError::InvalidInput("version_id not set".into()))?;

        let file_list = self.file_names_for_metadata(&files);
        let archive_hash = self.calculate_archive_hash(&archive_path)?;
        let archive_size = fs::metadata(&archive_path)
            .map(|metadata| metadata.len())
            .unwrap_or_default();

        info!("[PACKAGER] Archive hash: {archive_hash}");

        let metadata = SaveMetadata {
            game_id: self.game_id.clone(),
            emulator_id: self.emulator_id.clone(),
            timestamp,
            version_id,
            file_list,
            hash: archive_hash.clone(),
            size_bytes: Some(archive_size),
            sha256: Some(archive_hash.clone()),
            source: Some("local".to_string()),
        };

        info!("[PACKAGER] Final metadata: {:?}", metadata);
        Ok(metadata)
    }

    pub fn package_save(
        &mut self,
        paths: Vec<PathBuf>,
        patterns: Vec<String>,
    ) -> Result<PackagedSave, PackagerError> {
        let files = self.collect_files(paths, patterns)?;
        let timestamp = self.current_timestamp()?;
        let file_list = self.file_names_for_metadata(&files);
        let version_id = Self::generate_version_id(timestamp, &file_list);

        self.timestamp = Some(timestamp);
        self.version_id = Some(version_id.clone());

        let archive_path = self.create_archive(files.clone())?;
        let metadata = self.generate_metadata(files)?;

        Ok(PackagedSave {
            archive_path: archive_path.to_string_lossy().to_string(),
            metadata,
        })
    }

    fn collect_from_directory(&self, dir: &Path, patterns: &[Pattern], files: &mut Vec<PathBuf>) {
        let mut stack = vec![dir.to_path_buf()];

        while let Some(current) = stack.pop() {
            let entries = match fs::read_dir(&current) {
                Ok(entries) => entries,
                Err(err) => {
                    warn!("[PACKAGER] Failed to read directory {:?}: {}", current, err);
                    continue;
                }
            };

            for entry in entries.flatten() {
                let path = entry.path();
                match entry.metadata() {
                    Ok(metadata) => {
                        if metadata.is_dir() {
                            stack.push(path);
                        } else if metadata.is_file() {
                            if self.matches_patterns(&path, patterns) {
                                files.push(path);
                            }
                        }
                    }
                    Err(err) => warn!("[PACKAGER] Failed to read metadata for {:?}: {}", path, err),
                }
            }
        }
    }

    fn matches_patterns(&self, path: &Path, patterns: &[Pattern]) -> bool {
        if patterns.is_empty() {
            return true;
        }

        patterns.iter().any(|pattern| pattern.matches_path(path))
    }

    fn entry_name(&self, path: &Path, index: usize) -> String {
        path.file_name()
            .and_then(|name| name.to_str())
            .map(|name| name.to_string())
            .unwrap_or_else(|| format!("file_{index}"))
    }

    fn file_names_for_metadata(&self, files: &[PathBuf]) -> Vec<String> {
        let mut names: Vec<String> = files.iter().map(|path| self.entry_name(path, 0)).collect();
        names.sort();
        names.dedup();
        names
    }

    fn current_timestamp(&self) -> Result<u64, PackagerError> {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|duration| duration.as_secs())
            .map_err(|err| PackagerError::Timestamp(err.to_string()))
    }

    fn generate_version_id(timestamp: u64, file_list: &[String]) -> String {
        let mut hasher = Sha256::new();
        let file_list_hash = format!("{:x}", Sha256::digest(file_list.join("|")));
        hasher.update(format!("{timestamp}{file_list_hash}").as_bytes());
        format!("{:x}", hasher.finalize())
    }

    fn calculate_archive_hash(&self, archive_path: &Path) -> Result<String, PackagerError> {
        let mut file =
            fs::File::open(archive_path).map_err(|err| PackagerError::Hash(err.to_string()))?;
        let mut buffer = Vec::new();
        file.read_to_end(&mut buffer)
            .map_err(|err| PackagerError::Hash(err.to_string()))?;

        Ok(format!("{:x}", Sha256::digest(buffer)))
    }
}
