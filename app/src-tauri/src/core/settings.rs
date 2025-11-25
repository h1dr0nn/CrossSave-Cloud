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
    #[serde(default)]
    pub cloud: CloudSettings,
    #[serde(default)]
    pub cloud_mode: CloudMode,
    #[serde(default)]
    pub self_host: SelfHostSettings,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            retention_limit: 10,
            auto_delete: true,
            cloud: CloudSettings::default(),
            cloud_mode: CloudMode::default(),
            self_host: SelfHostSettings::default(),
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum CloudMode {
    Official,
    SelfHost,
    Off,
}

impl Default for CloudMode {
    fn default() -> Self {
        CloudMode::Official
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CloudSettings {
    pub enabled: bool,
    pub base_url: String,
    pub api_key: String,
    pub device_id: String,
    #[serde(default)]
    pub device_name: String,
    #[serde(default)]
    pub platform: String,
    #[serde(default)]
    pub user_id: String,
    pub timeout_seconds: u64,
    #[serde(default)]
    pub has_registered_device: bool,
}

impl Default for CloudSettings {
    fn default() -> Self {
        Self {
            enabled: false,
            base_url: "https://api.crosssave.local".to_string(),
            api_key: String::new(),
            device_id: String::new(),
            device_name: String::new(),
            platform: String::new(),
            user_id: String::new(),
            timeout_seconds: 30,
            has_registered_device: false,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SelfHostSettings {
    pub id_server: String,
    pub relay_server: String,
    pub api_server: String,
    pub access_key: String,
}

impl Default for SelfHostSettings {
    fn default() -> Self {
        Self {
            id_server: String::new(),
            relay_server: String::new(),
            api_server: String::new(),
            access_key: String::new(),
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
