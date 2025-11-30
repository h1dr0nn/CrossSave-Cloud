use anyhow::{anyhow, Result};
use aws_config::BehaviorVersion;
use aws_credential_types::Credentials;
use aws_sdk_s3::{
    config::Region,
    presigning::PresigningConfig,
    primitives::ByteStream,
    Client,
};
use std::time::Duration;

#[derive(Clone)]
pub struct S3Client {
    client: Client,
    bucket: String,
}

impl S3Client {
    pub async fn new(
        endpoint: &str,
        region: &str,
        access_key: &str,
        secret_key: &str,
        bucket: &str,
    ) -> Result<Self> {
        let credentials = Credentials::new(
            access_key,
            secret_key,
            None,
            None,
            "static",
        );

        let config = aws_config::defaults(BehaviorVersion::latest())
            .endpoint_url(endpoint)
            .region(Region::new(region.to_string()))
            .credentials_provider(credentials)
            .load()
            .await;

        let s3_config = aws_sdk_s3::config::Builder::from(&config)
            .force_path_style(true) // Required for MinIO
            .build();

        let client = Client::from_conf(s3_config);

        Ok(Self {
            client,
            bucket: bucket.to_string(),
        })
    }

    /// Put object to S3
    pub async fn put_object(&self, key: &str, data: Vec<u8>) -> Result<()> {
        self.client
            .put_object()
            .bucket(&self.bucket)
            .key(key)
            .body(ByteStream::from(data))
            .content_type("application/json")
            .send()
            .await?;

        Ok(())
    }

    /// Get object from S3
    pub async fn get_object(&self, key: &str) -> Result<Vec<u8>> {
        let response = self.client
            .get_object()
            .bucket(&self.bucket)
            .key(key)
            .send()
            .await?;

        let data = response.body.collect().await?;
        Ok(data.into_bytes().to_vec())
    }

    /// Check if object exists
    pub async fn head_object(&self, key: &str) -> Result<bool> {
        match self.client
            .head_object()
            .bucket(&self.bucket)
            .key(key)
            .send()
            .await
        {
            Ok(_) => Ok(true),
            Err(_) => Ok(false),
        }
    }

    /// Generate presigned PUT URL
    pub async fn presign_put(&self, key: &str, ttl_seconds: u64) -> Result<String> {
        let presigning_config = PresigningConfig::builder()
            .expires_in(Duration::from_secs(ttl_seconds))
            .build()?;

        let presigned = self.client
            .put_object()
            .bucket(&self.bucket)
            .key(key)
            .content_type("application/zip")
            .presigned(presigning_config)
            .await?;

        Ok(presigned.uri().to_string())
    }

    /// Generate presigned GET URL
    pub async fn presign_get(&self, key: &str, ttl_seconds: u64) -> Result<String> {
        let presigning_config = PresigningConfig::builder()
            .expires_in(Duration::from_secs(ttl_seconds))
            .build()?;

        let presigned = self.client
            .get_object()
            .bucket(&self.bucket)
            .key(key)
            .presigned(presigning_config)
            .await?;

        Ok(presigned.uri().to_string())
    }

    /// Ensure bucket exists (create if not)
    pub async fn ensure_bucket(&self) -> Result<()> {
        match self.client
            .head_bucket()
            .bucket(&self.bucket)
            .send()
            .await
        {
            Ok(_) => {
                tracing::info!("Bucket '{}' already exists", self.bucket);
                Ok(())
            }
            Err(_) => {
                tracing::info!("Creating bucket '{}'", self.bucket);
                self.client
                    .create_bucket()
                    .bucket(&self.bucket)
                    .send()
                    .await?;
                Ok(())
            }
        }
    }
}
