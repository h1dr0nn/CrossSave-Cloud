pub mod jwt;

use crate::types::Claims;
use axum::{
    async_trait,
    extract::FromRequestParts,
    http::{request::Parts, StatusCode},
    response::{IntoResponse, Response},
    Json, RequestPartsExt,
};
use axum_extra::{
    headers::{authorization::Bearer, Authorization},
    TypedHeader,
};
use serde_json::json;

pub use jwt::{hash_password, sign_jwt, verify_jwt, verify_password};

/// Authentication context extracted from request
#[derive(Debug, Clone)]
pub struct AuthContext {
    pub user_id: String,
    pub device_id: Option<String>,
}

/// Error response for authentication failures
pub struct AuthError;

impl IntoResponse for AuthError {
    fn into_response(self) -> Response {
        let body = Json(json!({
            "error": "unauthorized"
        }));
        (StatusCode::UNAUTHORIZED, body).into_response()
    }
}

/// Extract authentication from request headers
#[async_trait]
impl<S> FromRequestParts<S> for AuthContext
where
    S: Send + Sync,
{
    type Rejection = AuthError;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        // Extract Authorization header
        let TypedHeader(Authorization(bearer)) = parts
            .extract::<TypedHeader<Authorization<Bearer>>>()
            .await
            .map_err(|_| AuthError)?;

        // Verify JWT token
        let claims = verify_jwt(bearer.token()).map_err(|_| AuthError)?;

        Ok(AuthContext {
            user_id: claims.user_id,
            device_id: claims.device_id,
        })
    }
}
