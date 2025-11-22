use std::{
    collections::HashMap,
    fs,
    path::{Path, PathBuf},
    sync::Mutex,
};

use serde::{Deserialize, Serialize};
use thiserror::Error;
use tracing::{debug, info};

#[derive(Debug, Error)]
pub enum ProfileError {
    #[error("profiles directory missing at {0}")]
    ProfilesDirectoryMissing(String),
    #[error("failed to read profiles directory {0}: {1}")]
    ProfilesDirectoryRead(String, String),
    #[error("failed to read profile file {0}: {1}")]
    ProfileFileRead(String, String),
    #[error("failed to parse profile file {0}: {1}")]
    ProfileParse(String, String),
    #[error("invalid profile: {0}")]
    InvalidProfile(String),
    #[error("io error: {0}")]
    Io(String),
    #[error("lock error: {0}")]
    Lock(String),
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EmulatorProfile {
    pub emulator_id: String,
    pub name: String,
    pub default_save_paths: Vec<String>,
    pub file_patterns: Vec<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct RawEmulatorProfile {
    emulator_id: String,
    name: String,
    default_save_paths: Vec<String>,
    file_patterns: Vec<String>,
}

#[derive(Debug)]
pub struct ProfileManager {
    default_dir: PathBuf,
    user_dir: PathBuf,
    cache: Mutex<HashMap<String, EmulatorProfile>>,
}

impl ProfileManager {
    pub fn new(default_dir: PathBuf, user_dir: PathBuf) -> Result<Self, ProfileError> {
        if !default_dir.exists() {
            return Err(ProfileError::ProfilesDirectoryMissing(
                default_dir.display().to_string(),
            ));
        }

        if let Some(parent) = user_dir.parent() {
            fs::create_dir_all(parent).map_err(|err| ProfileError::Io(err.to_string()))?;
        }
        fs::create_dir_all(&user_dir).map_err(|err| ProfileError::Io(err.to_string()))?;

        let mut manager = Self {
            default_dir,
            user_dir,
            cache: Mutex::new(HashMap::new()),
        };

        manager.reload()?;
        Ok(manager)
    }

    pub fn list_profiles(&self) -> Result<Vec<EmulatorProfile>, ProfileError> {
        let guard = self
            .cache
            .lock()
            .map_err(|err| ProfileError::Lock(err.to_string()))?;
        let mut profiles: Vec<EmulatorProfile> = guard.values().cloned().collect();
        profiles.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));
        Ok(profiles)
    }

    pub fn get_profile(&self, emulator_id: &str) -> Result<Option<EmulatorProfile>, ProfileError> {
        let guard = self
            .cache
            .lock()
            .map_err(|err| ProfileError::Lock(err.to_string()))?;
        Ok(guard.get(emulator_id).cloned())
    }

    pub fn save_profile(
        &mut self,
        profile: EmulatorProfile,
    ) -> Result<EmulatorProfile, ProfileError> {
        Self::validate_profile(&profile)?;
        self.persist_profile(&profile)?;
        self.reload()?;
        info!("[PROFILE] Saved profile {}", profile.emulator_id);
        Ok(profile)
    }

    pub fn delete_profile(&mut self, emulator_id: &str) -> Result<(), ProfileError> {
        let user_profile = self.user_dir.join(format!("{emulator_id}.json"));
        if user_profile.exists() {
            fs::remove_file(&user_profile).map_err(|err| ProfileError::Io(err.to_string()))?;
            info!("[PROFILE] Deleted user profile {emulator_id}");
        } else {
            return Err(ProfileError::InvalidProfile(format!(
                "profile {emulator_id} is read-only"
            )));
        }

        self.reload()?;
        Ok(())
    }

    fn reload(&mut self) -> Result<(), ProfileError> {
        let mut merged: HashMap<String, EmulatorProfile> = HashMap::new();
        let default_profiles = self.load_dir(&self.default_dir)?;
        for profile in default_profiles {
            merged.insert(profile.emulator_id.clone(), profile);
        }

        let user_profiles = self.load_dir(&self.user_dir)?;
        for profile in user_profiles {
            merged.insert(profile.emulator_id.clone(), profile);
        }

        let mut guard = self
            .cache
            .lock()
            .map_err(|err| ProfileError::Lock(err.to_string()))?;
        *guard = merged;
        Ok(())
    }

    fn load_dir(&self, dir: &Path) -> Result<Vec<EmulatorProfile>, ProfileError> {
        if !dir.exists() {
            return Ok(Vec::new());
        }

        let entries = fs::read_dir(dir).map_err(|err| {
            ProfileError::ProfilesDirectoryRead(dir.display().to_string(), err.to_string())
        })?;

        let mut profiles: Vec<EmulatorProfile> = Vec::new();

        for entry in entries {
            let entry = entry.map_err(|err| {
                ProfileError::ProfilesDirectoryRead(dir.display().to_string(), err.to_string())
            })?;
            let path = entry.path();

            if path.extension().and_then(|ext| ext.to_str()) != Some("json") {
                continue;
            }

            debug!("[PROFILE] Loading profile from {:?}", path);

            let content = fs::read_to_string(&path).map_err(|err| {
                ProfileError::ProfileFileRead(path.display().to_string(), err.to_string())
            })?;

            let raw_profile: RawEmulatorProfile =
                serde_json::from_str(&content).map_err(|err| {
                    ProfileError::ProfileParse(path.display().to_string(), err.to_string())
                })?;

            let normalized_paths = self.normalize_paths(&raw_profile.default_save_paths)?;

            profiles.push(EmulatorProfile {
                emulator_id: raw_profile.emulator_id,
                name: raw_profile.name,
                default_save_paths: normalized_paths,
                file_patterns: raw_profile.file_patterns,
            });
        }

        Ok(profiles)
    }

    fn persist_profile(&self, profile: &EmulatorProfile) -> Result<(), ProfileError> {
        let raw = RawEmulatorProfile {
            emulator_id: profile.emulator_id.clone(),
            name: profile.name.clone(),
            default_save_paths: profile.default_save_paths.clone(),
            file_patterns: profile.file_patterns.clone(),
        };

        let json = serde_json::to_string_pretty(&raw)
            .map_err(|err| ProfileError::ProfileParse("serialize".into(), err.to_string()))?;
        let destination = self.user_dir.join(format!("{}.json", profile.emulator_id));
        fs::write(&destination, json).map_err(|err| ProfileError::Io(err.to_string()))?;
        Ok(())
    }

    fn normalize_paths(&self, paths: &[String]) -> Result<Vec<String>, ProfileError> {
        let mut validated: Vec<String> = Vec::new();

        for path in paths {
            let expanded = self.expand_home(path)?;
            validated.push(expanded.to_string_lossy().to_string());
        }

        Ok(validated)
    }

    fn expand_home(&self, path: &str) -> Result<PathBuf, ProfileError> {
        if path == "~" || path.starts_with("~/") {
            match std::env::var("HOME") {
                Ok(home) => {
                    let suffix = path.trim_start_matches('~').trim_start_matches('/');
                    let expanded = if suffix.is_empty() {
                        PathBuf::from(home)
                    } else {
                        Path::new(&home).join(suffix)
                    };
                    Ok(expanded)
                }
                Err(_) => {
                    // If HOME is not set (e.g. Android), just return the path as is
                    // This prevents the app from crashing, even if the path is invalid
                    debug!("[PROFILE] HOME not set, keeping path as is: {}", path);
                    Ok(PathBuf::from(path))
                }
            }
        } else {
            Ok(PathBuf::from(path))
        }
    }

    fn validate_profile(profile: &EmulatorProfile) -> Result<(), ProfileError> {
        if profile.name.trim().is_empty() {
            return Err(ProfileError::InvalidProfile("name cannot be empty".into()));
        }

        if profile.emulator_id.trim().is_empty() {
            return Err(ProfileError::InvalidProfile(
                "emulator_id cannot be empty".into(),
            ));
        }

        if profile.default_save_paths.is_empty() {
            return Err(ProfileError::InvalidProfile(
                "default_save_paths cannot be empty".into(),
            ));
        }

        if profile.file_patterns.is_empty() {
            return Err(ProfileError::InvalidProfile(
                "file_patterns cannot be empty".into(),
            ));
        }

        Ok(())
    }
}
