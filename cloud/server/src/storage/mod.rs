pub mod s3_client;

use crate::types::{UserDevices, UserMetadata, UserSaveMetadata};
use anyhow::Result;
pub use s3_client::S3Client;
use serde::{de::DeserializeOwned, Serialize};

/// Storage key utilities matching official worker format

pub fn get_user_base_key(user_id: &str) -> String {
    format!("users/{}/", user_id)
}

pub fn get_user_metadata_key(user_id: &str) -> String {
    format!("{}metadata.json", get_user_base_key(user_id))
}

pub fn get_user_devices_key(user_id: &str) -> String {
    format!("{}devices.json", get_user_base_key(user_id))
}

pub fn get_save_metadata_key(user_id: &str) -> String {
    format!("{}save_metadata.json", get_user_base_key(user_id))
}

pub fn get_save_object_key(user_id: &str, game_id: &str, version_id: &str) -> String {
    format!("{}saves/{}/{}.zip", get_user_base_key(user_id), game_id, version_id)
}

/// Read JSON object from S3
pub async fn read_json<T: DeserializeOwned>(client: &S3Client, key: &str) -> Result<Option<T>> {
    match client.get_object(key).await {
        Ok(data) => {
            let parsed: T = serde_json::from_slice(&data)?;
            Ok(Some(parsed))
        }
        Err(_) => Ok(None), // Object doesn't exist
    }
}

/// Write JSON object to S3
pub async fn write_json<T: Serialize>(client: &S3Client, key: &str, data: &T) -> Result<()> {
    let json = serde_json::to_vec_pretty(data)?;
    client.put_object(key, json).await?;
    Ok(())
}

/// Load user metadata or return default
pub async fn load_user_metadata(client: &S3Client, user_id: &str) -> Result<Option<UserMetadata>> {
    let key = get_user_metadata_key(user_id);
    read_json(client, &key).await
}

/// Save user metadata
pub async fn save_user_metadata(client: &S3Client, metadata: &UserMetadata) -> Result<()> {
    let key = get_user_metadata_key(&metadata.user_id);
    write_json(client, &key, metadata).await
}

/// Load user devices or return default
pub async fn load_user_devices(client: &S3Client, user_id: &str) -> Result<UserDevices> {
    let key = get_user_devices_key(user_id);
    Ok(read_json(client, &key).await?.unwrap_or_default())
}

/// Save user devices
pub async fn save_user_devices(client: &S3Client, user_id: &str, devices: &UserDevices) -> Result<()> {
    let key = get_user_devices_key(user_id);
    write_json(client, &key, devices).await
}

/// Load save metadata or return default
pub async fn load_save_metadata(client: &S3Client, user_id: &str) -> Result<UserSaveMetadata> {
    let key = get_save_metadata_key(user_id);
    Ok(read_json(client, &key).await?.unwrap_or_default())
}

/// Save save metadata
pub async fn save_save_metadata(client: &S3Client, user_id: &str, metadata: &UserSaveMetadata) -> Result<()> {
    let key = get_save_metadata_key(user_id);
    write_json(client, &key, metadata).await
}
