use axum::{extract::State, http::StatusCode, Json};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

use crate::{
    auth::{hash_password, sign_jwt, verify_password},
    storage::{
        load_user_devices, load_user_metadata, save_user_devices, save_user_metadata,
        S3Client,
    },
    types::{Claims, Device, UserDevices, UserMetadata},
    validation::{validate_device_id, validate_email},
};

const SESSION_TTL_SECONDS: i64 = 60 * 60 * 24 * 7; // 7 days

#[derive(Debug, Deserialize)]
pub struct SignupRequest {
    pub email: String,
    pub password: String,
    #[serde(default)]
    pub device_id: Option<String>,
    #[serde(default)]
    pub device_name: Option<String>,
    #[serde(default)]
    pub platform: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
    #[serde(default)]
    pub device_id: Option<String>,
    #[serde(default)]
    pub device_name: Option<String>,
    #[serde(default)]
    pub platform: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct AuthResponse {
    pub ok: bool,
    pub user_id: String,
    pub token: String,
    pub exp: i64,
    pub email: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub device_id: Option<String>,
}

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
) -> Result<Option<UserMetadata>, anyhow::Error> {
    // Since we don't have email index, we need to scan
    // For now, we'll use a simple approach: hash email to create user_id lookup key
    // This is a simplified version - production should use proper indexing
    
    // For this implementation, we'll store a mapping file
    let lookup_key = format!("email_lookup/{}.json", email);
    
    match crate::storage::read_json::<UserMetadata>(client, &lookup_key).await {
        Ok(Some(metadata)) => Ok(Some(metadata)),
        Ok(None) => Ok(None),
        Err(_) => Ok(None),
    }
}

/// Save email lookup
async fn save_email_lookup(
    client: &S3Client,
    email: &str,
    user_id: &str,
) -> Result<(), anyhow::Error> {
    let lookup_key = format!("email_lookup/{}.json", email);
    let data = json!({ "user_id": user_id, "email": email });
    crate::storage::write_json(client, &lookup_key, &data).await?;
    Ok(())
}

/// Handle signup
pub async fn handle_signup(
    State(client): State<S3Client>,
    Json(req): Json<SignupRequest>,
) -> Result<Json<AuthResponse>, (StatusCode, Json<Value>)> {
    // Validate inputs
    let email = normalize_email(&req.email);
    if !validate_email(&email) {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(json!({ "error": "invalid_email" })),
        ));
    }

    if req.password.len() < 8 {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(json!({ "error": "weak_password" })),
        ));
    }

    if !validate_device_id(&req.device_id) {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(json!({ "error": "invalid_device_id" })),
        ));
    }

    // Check if user exists
    match get_user_by_email(&client, &email).await {
        Ok(Some(_)) => {
            return Err((
                StatusCode::BAD_REQUEST,
                Json(json!({ "error": "email_already_registered" })),
            ));
        }
        Ok(None) => {}
        Err(e) => {
            tracing::error!("Failed to check existing user: {}", e);
            return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": "internal_error" })),
            ));
        }
    }

    // Create user
    let user_id = uuid::Uuid::new_v4().to_string();
    let password_hash = hash_password(&req.password).map_err(|e| {
        tracing::error!("Failed to hash password: {}", e);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "error": "internal_error" })),
        )
    })?;

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
    save_user_metadata(&client, &metadata).await.map_err(|e| {
        tracing::error!("Failed to save user metadata: {}", e);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "error": "internal_error" })),
        )
    })?;

    // Save email lookup
    save_email_lookup(&client, &email, &user_id).await.map_err(|e| {
        tracing::error!("Failed to save email lookup: {}", e);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "error": "internal_error" })),
        )
    })?;

    // Register device if provided
    if let Some(device_id) = &req.device_id {
        let device = Device {
            device_id: device_id.clone(),
            platform: normalize_platform(req.platform.as_deref()),
            device_name: normalize_device_name(req.device_name.as_deref()),
            last_seen: now,
        };

        let devices = UserDevices {
            devices: vec![device],
        };

        save_user_devices(&client, &user_id, &devices).await.map_err(|e| {
            tracing::error!("Failed to save user devices: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": "internal_error" })),
            )
        })?;
    }

    // Generate JWT
    let exp = now + SESSION_TTL_SECONDS;
    let claims = Claims {
        user_id: user_id.clone(),
        device_id: req.device_id.clone(),
        exp,
    };

    let token = sign_jwt(&claims).map_err(|e| {
        tracing::error!("Failed to sign JWT: {}", e);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "error": "internal_error" })),
        )
    })?;

    Ok(Json(AuthResponse {
        ok: true,
        user_id,
        token,
        exp,
        email,
        device_id: req.device_id,
    }))
}

/// Handle login
pub async fn handle_login(
    State(client): State<S3Client>,
    Json(req): Json<LoginRequest>,
) -> Result<Json<AuthResponse>, (StatusCode, Json<Value>)> {
    // Validate inputs
    let email = normalize_email(&req.email);
    if !validate_email(&email) || req.password.is_empty() {
        return Err((
            StatusCode::UNAUTHORIZED,
            Json(json!({ "error": "invalid_credentials" })),
        ));
    }

    if !validate_device_id(&req.device_id) {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(json!({ "error": "invalid_device_id" })),
        ));
    }

    // Get user
    let user = get_user_by_email(&client, &email).await.map_err(|e| {
        tracing::error!("Failed to get user: {}", e);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "error": "internal_error" })),
        )
    })?;

    let user = user.ok_or_else(|| {
        (
            StatusCode::UNAUTHORIZED,
            Json(json!({ "error": "invalid_credentials" })),
        )
    })?;

    // Verify password
    let valid = verify_password(&req.password, &user.password_hash).map_err(|e| {
        tracing::error!("Failed to verify password: {}", e);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "error": "internal_error" })),
        )
    })?;

    if !valid {
        return Err((
            StatusCode::UNAUTHORIZED,
            Json(json!({ "error": "invalid_credentials" })),
        ));
    }

    // Update device if provided
    let now = chrono::Utc::now().timestamp();
    if let Some(device_id) = &req.device_id {
        let mut devices = load_user_devices(&client, &user.user_id).await.map_err(|e| {
            tracing::error!("Failed to load devices: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": "internal_error" })),
            )
        })?;

        // Update or add device
        if let Some(existing) = devices.devices.iter_mut().find(|d| d.device_id == *device_id) {
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

        save_user_devices(&client, &user.user_id, &devices).await.map_err(|e| {
            tracing::error!("Failed to save devices: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": "internal_error" })),
            )
        })?;
    }

    // Generate JWT
    let exp = now + SESSION_TTL_SECONDS;
    let claims = Claims {
        user_id: user.user_id.clone(),
        device_id: req.device_id.clone(),
        exp,
    };

    let token = sign_jwt(&claims).map_err(|e| {
        tracing::error!("Failed to sign JWT: {}", e);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "error": "internal_error" })),
        )
    })?;

    Ok(Json(AuthResponse {
        ok: true,
        user_id: user.user_id,
        token,
        exp,
        email: user.email,
        device_id: req.device_id,
    }))
}
