use axum::{extract::State, Json};
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::{
    auth::AuthContext,
    error::AppError,
    services::device::DeviceService,
    storage::S3Client,
    types::Device,
};

#[derive(Debug, Deserialize)]
pub struct RegisterDeviceRequest {
    pub device_id: String,
    #[serde(default)]
    pub device_name: Option<String>,
    #[serde(default)]
    pub platform: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct RemoveDeviceRequest {
    pub device_id: String,
}

#[derive(Debug, Serialize)]
pub struct DeviceResponse {
    pub ok: bool,
    pub device: Device,
}

#[derive(Debug, Serialize)]
pub struct DeviceListResponse {
    pub ok: bool,
    pub devices: Vec<Device>,
}

/// Handle device registration
pub async fn handle_register_device(
    auth: AuthContext,
    State(client): State<S3Client>,
    Json(req): Json<RegisterDeviceRequest>,
) -> Result<Json<DeviceResponse>, AppError> {
    let response = DeviceService::register_device(&client, &auth, req).await?;
    Ok(Json(response))
}

/// Handle device list
pub async fn handle_list_devices(
    auth: AuthContext,
    State(client): State<S3Client>,
) -> Result<Json<DeviceListResponse>, AppError> {
    let response = DeviceService::list_devices(&client, &auth).await?;
    Ok(Json(response))
}

/// Handle device removal
pub async fn handle_remove_device(
    auth: AuthContext,
    State(client): State<S3Client>,
    Json(req): Json<RemoveDeviceRequest>,
) -> Result<Json<Value>, AppError> {
    let response = DeviceService::remove_device(&client, &auth, req).await?;
    Ok(Json(response))
}
