use std::{fs, path::PathBuf, sync::Mutex};

use serde::{Deserialize, Serialize};
use thiserror::Error;
use tracing::{info, warn};

const MIN_RETENTION: usize = 5;
const MAX_RETENTION: usize = 20;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AppSettings {
    pub retention_limit: usize,
    pub auto_delete: bool,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            retention_limit: 10,
            auto_delete: true,
        }
    }
}

#[derive(Debug, Error)]
pub enum SettingsError {
    #[error("io error: {0}")]
    Io(String),
    #[error("serialization error: {0}")]
    Serialization(String),
    #[error("invalid retention limit {0}, expected {1}-{2}")]
    InvalidRetention(usize, usize, usize),
    #[error("lock error: {0}")]
    Lock(String),
}

pub struct SettingsManager {
    path: PathBuf,
    state: Mutex<AppSettings>,
}

impl SettingsManager {
    pub fn new(path: PathBuf) -> Result<Self, SettingsError> {
        if let Some(parent) = path.parent() {
            if let Err(err) = fs::create_dir_all(parent) {
                return Err(SettingsError::Io(err.to_string()));
            }
        }

        let loaded = match fs::read_to_string(&path) {
            Ok(content) => match serde_json::from_str::<AppSettings>(&content) {
                Ok(settings) => settings,
                Err(err) => {
                    warn!("[SETTINGS] Failed to parse settings file: {err}. Using defaults");
                    AppSettings::default()
                }
            },
            Err(_) => AppSettings::default(),
        };

        let validated = Self::validate(loaded)?;
        Ok(Self {
            path,
            state: Mutex::new(validated),
        })
    }

    pub fn get_settings(&self) -> Result<AppSettings, SettingsError> {
        let guard = self
            .state
            .lock()
            .map_err(|err| SettingsError::Lock(err.to_string()))?;
        Ok(guard.clone())
    }

    pub fn update_settings(&self, settings: AppSettings) -> Result<AppSettings, SettingsError> {
        let validated = Self::validate(settings)?;
        let mut guard = self
            .state
            .lock()
            .map_err(|err| SettingsError::Lock(err.to_string()))?;
        *guard = validated.clone();

        let json = serde_json::to_string_pretty(&validated)
            .map_err(|err| SettingsError::Serialization(err.to_string()))?;
        fs::write(&self.path, json).map_err(|err| SettingsError::Io(err.to_string()))?;

        info!(
            "[SETTINGS] Updated retention to {} (auto_delete={})",
            validated.retention_limit, validated.auto_delete
        );
        Ok(validated)
    }

    fn validate(settings: AppSettings) -> Result<AppSettings, SettingsError> {
        if settings.retention_limit < MIN_RETENTION || settings.retention_limit > MAX_RETENTION {
            return Err(SettingsError::InvalidRetention(
                settings.retention_limit,
                MIN_RETENTION,
                MAX_RETENTION,
            ));
        }

        Ok(settings)
    }
}

pub fn default_retention_bounds() -> (usize, usize) {
    (MIN_RETENTION, MAX_RETENTION)
}
