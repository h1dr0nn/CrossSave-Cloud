use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use crosssave_selfhost_server::{
    routes::auth::{AuthResponse, LoginRequest, SignupRequest},
    storage::S3Client,
};
use tower::ServiceExt; // for oneshot

// Helper to create a test app
async fn create_app() -> axum::Router {
    // Use the helper from lib.rs which sets up a dummy S3 client
    crosssave_selfhost_server::create_test_router().await
}

#[tokio::test]
async fn test_signup_validation() {
    let app = create_app().await;

    let payload = SignupRequest {
        email: "invalid-email".to_string(),
        password: "short".to_string(),
        device_id: None,
        device_name: None,
        platform: None,
    };

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/signup")
                .header("Content-Type", "application/json")
                .body(Body::from(serde_json::to_string(&payload).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    // Should fail validation
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_login_validation() {
    let app = create_app().await;

    let payload = LoginRequest {
        email: "test@example.com".to_string(),
        password: "".to_string(), // Empty password
        device_id: None,
        device_name: None,
        platform: None,
    };

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/login")
                .header("Content-Type", "application/json")
                .body(Body::from(serde_json::to_string(&payload).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    // Should fail validation
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}
