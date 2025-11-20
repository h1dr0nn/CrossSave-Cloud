use std::{
    env, fs,
    path::{Path, PathBuf},
    sync::OnceLock,
};

use serde::{Deserialize, Serialize};
use thiserror::Error;
use tracing::{debug, info, warn};

static PROFILES: OnceLock<Vec<EmulatorProfile>> = OnceLock::new();

#[derive(Debug, Error)]
pub enum ProfileError {
    #[error("profiles already loaded")]
    ProfilesAlreadyLoaded,
    #[error("profiles not loaded")]
    ProfilesNotLoaded,
    #[error("profiles directory missing at {0}")]
    ProfilesDirectoryMissing(String),
    #[error("failed to read profiles directory {0}: {1}")]
    ProfilesDirectoryRead(String, String),
    #[error("failed to read profile file {0}: {1}")]
    ProfileFileRead(String, String),
    #[error("failed to parse profile file {0}: {1}")]
    ProfileParse(String, String),
    #[error("home directory not found for path {0}")]
    HomeDirectoryUnavailable(String),
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct RawEmulatorProfile {
    emulator_id: String,
    name: String,
    default_save_paths: Vec<String>,
    file_patterns: Vec<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EmulatorProfile {
    pub emulator_id: String,
    pub name: String,
    pub default_save_paths: Vec<String>,
    pub file_patterns: Vec<String>,
}

pub fn load_profiles() -> Result<(), ProfileError> {
    if PROFILES.get().is_some() {
        return Err(ProfileError::ProfilesAlreadyLoaded);
    }

    let profiles_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("resources")
        .join("profiles");

    if !profiles_dir.exists() {
        return Err(ProfileError::ProfilesDirectoryMissing(
            profiles_dir.display().to_string(),
        ));
    }

    let entries = fs::read_dir(&profiles_dir).map_err(|err| {
        ProfileError::ProfilesDirectoryRead(profiles_dir.display().to_string(), err.to_string())
    })?;

    let mut profiles: Vec<EmulatorProfile> = Vec::new();

    for entry in entries {
        let entry = entry.map_err(|err| {
            ProfileError::ProfilesDirectoryRead(profiles_dir.display().to_string(), err.to_string())
        })?;
        let path = entry.path();

        if path.extension().and_then(|ext| ext.to_str()) != Some("json") {
            continue;
        }

        debug!("[PROFILE] Loading profile from {:?}", path);

        let content = fs::read_to_string(&path).map_err(|err| {
            ProfileError::ProfileFileRead(path.display().to_string(), err.to_string())
        })?;

        let raw_profile: RawEmulatorProfile = serde_json::from_str(&content).map_err(|err| {
            ProfileError::ProfileParse(path.display().to_string(), err.to_string())
        })?;

        let validated_paths = normalize_paths(&raw_profile.default_save_paths)?;

        profiles.push(EmulatorProfile {
            emulator_id: raw_profile.emulator_id,
            name: raw_profile.name,
            default_save_paths: validated_paths,
            file_patterns: raw_profile.file_patterns,
        });
    }

    PROFILES
        .set(profiles)
        .map_err(|_| ProfileError::ProfilesAlreadyLoaded)?;

    if let Some(loaded) = PROFILES.get() {
        info!("[PROFILE] Loaded {} emulator profiles", loaded.len());
    }

    Ok(())
}

pub fn get_profiles() -> Result<Vec<EmulatorProfile>, ProfileError> {
    match PROFILES.get() {
        Some(profiles) => Ok(profiles.clone()),
        None => Err(ProfileError::ProfilesNotLoaded),
    }
}

pub fn get_profile_by_id(emulator_id: &str) -> Result<Option<EmulatorProfile>, ProfileError> {
    let profiles = PROFILES.get().ok_or(ProfileError::ProfilesNotLoaded)?;
    Ok(profiles
        .iter()
        .find(|profile| profile.emulator_id == emulator_id)
        .cloned())
}

fn normalize_paths(paths: &[String]) -> Result<Vec<String>, ProfileError> {
    let mut validated: Vec<String> = Vec::new();

    for path in paths {
        let expanded = expand_home(path)?;

        match fs::metadata(&expanded) {
            Ok(_) => validated.push(expanded.to_string_lossy().to_string()),
            Err(err) => warn!(
                "[PROFILE] Skipping path {:?} - validation failed: {}",
                expanded, err
            ),
        }
    }

    Ok(validated)
}

fn expand_home(path: &str) -> Result<PathBuf, ProfileError> {
    if path == "~" || path.starts_with("~/") {
        let home = env::var("HOME")
            .map_err(|_| ProfileError::HomeDirectoryUnavailable(path.to_string()))?;
        let suffix = path.trim_start_matches('~').trim_start_matches('/');
        let expanded = if suffix.is_empty() {
            PathBuf::from(home)
        } else {
            Path::new(&home).join(suffix)
        };
        Ok(expanded)
    } else {
        Ok(PathBuf::from(path))
    }
}
