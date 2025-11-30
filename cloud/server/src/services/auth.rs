use crate::{
    auth::{hash_password, sign_jwt, verify_password},
    error::AppError,
    routes::auth::{AuthResponse, LoginRequest, SignupRequest},
    storage::{
        load_user_devices, save_user_devices, save_user_metadata, S3Client,
    },
    types::{Claims, Device, UserDevices, UserMetadata},
    validation::{validate_device_id, validate_email},
};
use serde_json::json;

const SESSION_TTL_SECONDS: i64 = 60 * 60 * 24 * 7; // 7 days

pub struct AuthService;

impl AuthService {
    /// Normalize email (lowercase, trim)
    fn normalize_email(email: &str) -> String {
        email.trim().to_lowercase()
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

    /// GET user by email
    async fn get_user_by_email(
        client: &S3Client,
        email: &str,
    ) -> Result<Option<UserMetadata>, AppError> {
        let lookup_key = format!("email_lookup/{}.json", email);
        crate::storage::read_json::<UserMetadata>(client, &lookup_key)
            .await
            .map_err(|e| AppError::InternalError(e.into()))
    }

    /// Save email lookup
    async fn save_email_lookup(
        client: &S3Client,
        email: &str,
        user_id: &str,
    ) -> Result<(), AppError> {
        let lookup_key = format!("email_lookup/{}.json", email);
        let data = json!({ "user_id": user_id, "email": email });
        crate::storage::write_json(client, &lookup_key, &data)
            .await
            .map_err(|e| AppError::InternalError(e.into()))
    }

    pub async fn signup(client: &S3Client, req: SignupRequest) -> Result<AuthResponse, AppError> {
        // Validate inputs
        let email = Self::normalize_email(&req.email);
        if !validate_email(&email) {
            return Err(AppError::InvalidInput("invalid_email".to_string()));
        }

        if req.password.len() < 8 {
            return Err(AppError::InvalidInput("weak_password".to_string()));
        }

        if !validate_device_id(&req.device_id) {
            return Err(AppError::InvalidInput("invalid_device_id".to_string()));
        }

        // Check if user exists
        if Self::get_user_by_email(client, &email).await?.is_some() {
            return Err(AppError::InvalidInput("email_already_registered".to_string()));
        }

        // Create user
        let user_id = uuid::Uuid::new_v4().to_string();
        let password_hash = hash_password(&req.password).map_err(|e| AppError::InternalError(e))?;

        let now = chrono::Utc::now().timestamp();
        let metadata = UserMetadata {
            user_id: user_id.clone(),
            email: email.clone(),
            password_hash,
            created_at: now,
            updated_at: now,
            devices: if req.device_id.is_some() { 1 } else { 0 },
        };

        // Save user metadata
        save_user_metadata(client, &metadata)
            .await
            .map_err(|e| AppError::InternalError(e.into()))?;

        // Save email lookup
        Self::save_email_lookup(client, &email, &user_id).await?;

        // Register device if provided
        if let Some(device_id) = &req.device_id {
            let device = Device {
                device_id: device_id.clone(),
                platform: Self::normalize_platform(req.platform.as_deref()),
                device_name: Self::normalize_device_name(req.device_name.as_deref()),
                last_seen: now,
            };

            let devices = UserDevices {
                devices: vec![device],
            };

            save_user_devices(client, &user_id, &devices)
                .await
                .map_err(|e| AppError::InternalError(e.into()))?;
        }

        // Generate JWT
        let exp = now + SESSION_TTL_SECONDS;
        let claims = Claims {
            user_id: user_id.clone(),
            device_id: req.device_id.clone(),
            exp,
        };

        let token = sign_jwt(&claims).map_err(|e| AppError::InternalError(e))?;

        Ok(AuthResponse {
            ok: true,
            user_id,
            token,
            exp,
            email,
            device_id: req.device_id,
        })
    }

    pub async fn login(client: &S3Client, req: LoginRequest) -> Result<AuthResponse, AppError> {
        // Validate inputs
        let email = Self::normalize_email(&req.email);
        if !validate_email(&email) || req.password.is_empty() {
            return Err(AppError::AuthError("invalid_credentials".to_string()));
        }

        if !validate_device_id(&req.device_id) {
            return Err(AppError::InvalidInput("invalid_device_id".to_string()));
        }

        // Get user
        let user = Self::get_user_by_email(client, &email)
            .await?
            .ok_or_else(|| AppError::AuthError("invalid_credentials".to_string()))?;

        // Verify password
        let valid = verify_password(&req.password, &user.password_hash)
            .map_err(|e| AppError::InternalError(e))?;

        if !valid {
            return Err(AppError::AuthError("invalid_credentials".to_string()));
        }

        // Update device if provided
        let now = chrono::Utc::now().timestamp();
        if let Some(device_id) = &req.device_id {
            let mut devices = load_user_devices(client, &user.user_id)
                .await
                .map_err(|e| AppError::InternalError(e.into()))?;

            // Update or add device
            if let Some(existing) = devices.devices.iter_mut().find(|d| d.device_id == *device_id) {
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

            save_user_devices(client, &user.user_id, &devices)
                .await
                .map_err(|e| AppError::InternalError(e.into()))?;
        }

        // Generate JWT
        let exp = now + SESSION_TTL_SECONDS;
        let claims = Claims {
            user_id: user.user_id.clone(),
            device_id: req.device_id.clone(),
            exp,
        };

        let token = sign_jwt(&claims).map_err(|e| AppError::InternalError(e))?;

        Ok(AuthResponse {
            ok: true,
            user_id: user.user_id,
            token,
            exp,
            email: user.email,
            device_id: req.device_id,
        })
    }
}
