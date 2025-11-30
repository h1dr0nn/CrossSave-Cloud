use crate::{
    auth::AuthContext,
    error::AppError,
    routes::device::{DeviceListResponse, DeviceResponse, RegisterDeviceRequest, RemoveDeviceRequest},
    storage::{load_user_devices, save_user_devices, S3Client},
    types::Device,
    validation::validate_device_id,
};
use serde_json::json;

pub struct DeviceService;

impl DeviceService {
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

    pub async fn register_device(
        client: &S3Client,
        auth: &AuthContext,
        req: RegisterDeviceRequest,
    ) -> Result<DeviceResponse, AppError> {
        let device_id = req.device_id.trim().to_string();

        if !validate_device_id(&Some(device_id.clone())) {
            return Err(AppError::InvalidInput("invalid_device_id".to_string()));
        }

        let mut devices = load_user_devices(client, &auth.user_id)
            .await
            .map_err(|e| AppError::InternalError(e.into()))?;

        let now = chrono::Utc::now().timestamp();

        // Update or add device
        if let Some(existing) = devices.devices.iter_mut().find(|d| d.device_id == device_id) {
            existing.last_seen = now;
            existing.platform = Self::normalize_platform(req.platform.as_deref());
            existing.device_name = Self::normalize_device_name(req.device_name.as_deref());
        } else {
            devices.devices.push(Device {
                device_id: device_id.clone(),
                platform: Self::normalize_platform(req.platform.as_deref()),
                device_name: Self::normalize_device_name(req.device_name.as_deref()),
                last_seen: now,
            });
        }

        save_user_devices(client, &auth.user_id, &devices)
            .await
            .map_err(|e| AppError::InternalError(e.into()))?;

        let device = devices
            .devices
            .iter()
            .find(|d| d.device_id == device_id)
            .unwrap()
            .clone();

        Ok(DeviceResponse { ok: true, device })
    }

    pub async fn list_devices(
        client: &S3Client,
        auth: &AuthContext,
    ) -> Result<DeviceListResponse, AppError> {
        let devices = load_user_devices(client, &auth.user_id)
            .await
            .map_err(|e| AppError::InternalError(e.into()))?;

        Ok(DeviceListResponse {
            ok: true,
            devices: devices.devices,
        })
    }

    pub async fn remove_device(
        client: &S3Client,
        auth: &AuthContext,
        req: RemoveDeviceRequest,
    ) -> Result<serde_json::Value, AppError> {
        let device_id = req.device_id.trim();

        if !validate_device_id(&Some(device_id.to_string())) {
            return Err(AppError::InvalidInput("invalid_device_id".to_string()));
        }

        // Prevent removing active device
        if let Some(ref active_device_id) = auth.device_id {
            if active_device_id == device_id {
                return Err(AppError::InvalidInput("cannot_remove_active_device".to_string()));
            }
        }

        let mut devices = load_user_devices(client, &auth.user_id)
            .await
            .map_err(|e| AppError::InternalError(e.into()))?;

        let original_count = devices.devices.len();
        devices.devices.retain(|d| d.device_id != device_id);

        if devices.devices.len() == original_count {
            return Err(AppError::NotFound("device_not_found".to_string()));
        }

        save_user_devices(client, &auth.user_id, &devices)
            .await
            .map_err(|e| AppError::InternalError(e.into()))?;

        Ok(json!({ "ok": true }))
    }
}
