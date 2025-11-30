use crosssave_selfhost_server::{
    auth, config, routes, storage, create_app
};
use std::net::SocketAddr;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Load environment variables
    dotenvy::dotenv().ok();

    // Initialize tracing
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "info,crosssave_selfhost_server=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    tracing::info!("Starting CrossSave Self-host Server...");

    // Load configuration
    let config = config::ServerConfig::from_env()?;
    tracing::info!("Server will bind to: {}", config.bind_address());

    // Initialize JWT auth
    auth::jwt::init_jwt(config.clone());

    // Initialize S3 client
    tracing::info!("Connecting to S3 endpoint: {}", config.s3_endpoint);
    let s3_client = storage::S3Client::new(
        &config.s3_endpoint,
        &config.s3_region,
        &config.s3_access_key,
        &config.s3_secret_key,
        &config.s3_bucket,
    )
    .await?;

    // Ensure bucket exists
    s3_client.ensure_bucket().await?;
    tracing::info!("S3 bucket '{}' ready", config.s3_bucket);

    // Create router with CORS and tracing
    let app = create_app(s3_client);

    // Bind and serve
    let addr: SocketAddr = config.bind_address().parse()?;
    tracing::info!("Server listening on {}", addr);
    
    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}
