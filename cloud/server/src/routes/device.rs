use axum::{extract::State, http::StatusCode, Json};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

use crate::{
    auth::AuthContext,
    storage::{load_user_devices, save_user_devices, S3Client},
    types::Device,
    validation::validate_device_id,
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

/// Normalize device name
fn normalize_device_name(name: Option<&str>) -> String {
    match name {
        Some(n) if !n.trim().is_empty() => n.trim().to_string(),
        _ => "Unknown Device".to_string(),
    }
}

/// Normalize platform
fn normalize_platform(platform: Option<&str>) -> String {
    match platform {
        Some(p) if !p.trim().is_empty() => p.trim().to_lowercase(),
        _ => "unknown".to_string(),
    }
}

/// Handle device registration
pub async fn handle_register_device(
    auth: AuthContext,
    State(client): State<S3Client>,
    Json(req): Json<RegisterDeviceRequest>,
) -> Result<Json<DeviceResponse>, (StatusCode, Json<Value>)> {
    let device_id = req.device_id.trim().to_string();
    
    if !validate_device_id(&Some(device_id.clone())) {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(json!({ "error": "invalid_device_id" })),
        ));
    }

    let mut devices = load_user_devices(&client, &auth.user_id)
        .await
        .map_err(|e| {
            tracing::error!("Failed to load devices: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": "internal_error" })),
            )
        })?;

    let now = chrono::Utc::now().timestamp();

    // Update or add device
    if let Some(existing) = devices.devices.iter_mut().find(|d| d.device_id == device_id) {
        existing.last_seen = now;
        existing.platform = normalize_platform(req.platform.as_deref());
        existing.device_name = normalize_device_name(req.device_name.as_deref());
    } else {
        devices.devices.push(Device {
            device_id: device_id.clone(),
            platform: normalize_platform(req.platform.as_deref()),
            device_name: normalize_device_name(req.device_name.as_deref()),
            last_seen: now,
        });
    }

    save_user_devices(&client, &auth.user_id, &devices)
        .await
        .map_err(|e| {
            tracing::error!("Failed to save devices: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": "internal_error" })),
            )
        })?;

    let device = devices
        .devices
        .iter()
        .find(|d| d.device_id == device_id)
        .unwrap()
        .clone();

    Ok(Json(DeviceResponse { ok: true, device }))
}

/// Handle device list
pub async fn handle_list_devices(
    auth: AuthContext,
    State(client): State<S3Client>,
) -> Result<Json<DeviceListResponse>, (StatusCode, Json<Value>)> {
    let devices = load_user_devices(&client, &auth.user_id)
        .await
        .map_err(|e| {
            tracing::error!("Failed to load devices: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": "internal_error" })),
            )
        })?;

    Ok(Json(DeviceListResponse {
        ok: true,
        devices: devices.devices,
    }))
}

/// Handle device removal
pub async fn handle_remove_device(
    auth: AuthContext,
    State(client): State<S3Client>,
    Json(req): Json<RemoveDeviceRequest>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let device_id = req.device_id.trim();
    
    if !validate_device_id(&Some(device_id.to_string())) {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(json!({ "error": "invalid_device_id" })),
        ));
    }

    // Prevent removing active device
    if let Some(ref active_device_id) = auth.device_id {
        if active_device_id == device_id {
            return Err((
                StatusCode::BAD_REQUEST,
                Json(json!({ "error": "cannot_remove_active_device" })),
            ));
        }
    }

    let mut devices = load_user_devices(&client, &auth.user_id)
        .await
        .map_err(|e| {
            tracing::error!("Failed to load devices: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": "internal_error" })),
            )
        })?;

    let original_count = devices.devices.len();
    devices.devices.retain(|d| d.device_id != device_id);

    if devices.devices.len() == original_count {
        return Err((
            StatusCode::NOT_FOUND,
            Json(json!({ "error": "device_not_found" })),
        ));
    }

    save_user_devices(&client, &auth.user_id, &devices)
        .await
        .map_err(|e| {
            tracing::error!("Failed to save devices: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": "internal_error" })),
            )
        })?;

    Ok(Json(json!({ "ok": true })))
}
