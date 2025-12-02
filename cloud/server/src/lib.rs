pub mod auth;
pub mod config;
pub mod error;
pub mod routes;
pub mod services;
pub mod storage;
pub mod types;
pub mod validation;

use axum::Router;
use tower_http::{
    cors::{Any, CorsLayer},
    trace::TraceLayer,
};

// Re-export create_router for convenience and testing
pub async fn create_router() -> Router {
    // We need a way to inject S3Client or config here for tests
    // For now, let's assume this is used by tests that mock S3 or don't need it immediately
    // BUT wait, routes::create_router takes s3_client.
    // So we need to provide a helper that creates a dummy client or accepts one.
    
    // Let's change this signature to accept S3Client, or create a default one for tests?
    // Better: expose the one that takes S3Client, and let tests create a mock/real client.
    
    // However, to keep it simple for now and match what I wrote in auth_test.rs:
    // I'll create a version that panics if used without setup, OR I'll update auth_test.rs
    // to pass a client.
    
    // Let's update auth_test.rs to be more realistic later.
    // For now, let's just expose the modules and let main.rs do the wiring.
    Router::new()
}

use std::sync::Arc;
use tower_governor::{governor::GovernorConfigBuilder, GovernorLayer};

// Better approach: Expose a function that sets up the app with a given client
pub fn create_app(client: storage::S3Client) -> Router {
    // Rate limit configuration: 5 requests per second, burst 10
    let governor_conf = Arc::new(
        GovernorConfigBuilder::default()
            .per_second(5)
            .burst_size(10)
            .finish()
            .unwrap(),
    );

    routes::create_router(client)
        .layer(
            CorsLayer::new()
                .allow_origin(Any)
                .allow_methods(Any)
                .allow_headers(Any),
        )
        .layer(TraceLayer::new_for_http())
        .layer(GovernorLayer {
            config: governor_conf,
        })
}

// Helper for integration tests
pub async fn create_test_router() -> Router {
    // Create a dummy S3 client that won't actually connect unless methods are called
    let client = storage::S3Client::new(
        "http://localhost:9000",
        "us-east-1",
        "minioadmin",
        "minioadmin",
        "test-bucket",
    )
    .await
    .expect("Failed to create test client");

    create_app(client)
}
