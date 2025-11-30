use serde::{Deserialize, Serialize};

/// User account metadata stored in S3
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserMetadata {
    pub user_id: String,
    pub email: String,
    pub password_hash: String,
    pub created_at: i64,
    pub updated_at: i64,
    pub devices: u32,
}

/// Device information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Device {
    pub device_id: String,
    pub platform: String,
    pub device_name: String,
    pub last_seen: i64,
}

/// List of devices for a user
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserDevices {
    pub devices: Vec<Device>,
}

impl Default for UserDevices {
    fn default() -> Self {
        Self {
            devices: Vec::new(),
        }
    }
}

/// Save version entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SaveVersion {
    pub version_id: String,
    pub game_id: String,
    pub size_bytes: u64,
    pub sha256: String,
    pub file_list: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub emulator_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub device_id: Option<String>,
    pub timestamp: i64,
}

/// User's save metadata (list of all versions)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserSaveMetadata {
    pub versions: Vec<SaveVersion>,
}

impl Default for UserSaveMetadata {
    fn default() -> Self {
        Self {
            versions: Vec::new(),
        }
    }
}

/// JWT claims
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Claims {
    pub user_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub device_id: Option<String>,
    pub exp: i64,
}

/// Upload request payload
#[derive(Debug, Clone, Deserialize)]
pub struct UploadPayload {
    pub game_id: String,
    pub version_id: String,
    pub size_bytes: u64,
    pub sha256: String,
    pub file_list: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub emulator_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub device_id: Option<String>,
}

/// Download request payload
#[derive(Debug, Clone, Deserialize)]
pub struct DownloadPayload {
    pub game_id: String,
    pub version_id: String,
}

/// Worker token claims (for upload proxy)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkerTokenClaims {
    pub user_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub device_id: Option<String>,
    pub r2_key: String,
    pub version_id: String,
    pub exp: i64,
}
