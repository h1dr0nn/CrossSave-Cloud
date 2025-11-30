use axum::{extract::State, Json};
use serde::{Deserialize, Serialize};

use crate::{
    error::AppError,
    services::auth::AuthService,
    storage::S3Client,
};

#[derive(Debug, Deserialize, Serialize)]
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

#[derive(Debug, Deserialize, Serialize)]
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

/// Handle signup
pub async fn handle_signup(
    State(client): State<S3Client>,
    Json(req): Json<SignupRequest>,
) -> Result<Json<AuthResponse>, AppError> {
    let response = AuthService::signup(&client, req).await?;
    Ok(Json(response))
}

/// Handle login
pub async fn handle_login(
    State(client): State<S3Client>,
    Json(req): Json<LoginRequest>,
) -> Result<Json<AuthResponse>, AppError> {
    let response = AuthService::login(&client, req).await?;
    Ok(Json(response))
}

