use crate::config::ServerConfig;
use crate::types::Claims;
use anyhow::{anyhow, Result};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use std::sync::OnceLock;

static CONFIG: OnceLock<ServerConfig> = OnceLock::new();

pub fn init_jwt(config: ServerConfig) {
    CONFIG.set(config).expect("JWT config already initialized");
}

/// Sign a JWT token
pub fn sign_jwt(claims: &Claims) -> Result<String> {
    let config = CONFIG
        .get()
        .ok_or_else(|| anyhow!("JWT config not initialized"))?;

    let token = encode(
        &Header::default(),
        claims,
        &EncodingKey::from_secret(config.jwt_secret.as_bytes()),
    )?;

    Ok(token)
}

/// Verify and decode a JWT token
pub fn verify_jwt(token: &str) -> Result<Claims> {
    let config = CONFIG
        .get()
        .ok_or_else(|| anyhow!("JWT config not initialized"))?;

    let token_data = decode::<Claims>(
        token,
        &DecodingKey::from_secret(config.jwt_secret.as_bytes()),
        &Validation::default(),
    )?;

    Ok(token_data.claims)
}

/// Hash a password using bcrypt
pub fn hash_password(password: &str) -> Result<String> {
    Ok(bcrypt::hash(password, bcrypt::DEFAULT_COST)?)
}

/// Verify a password against a hash
pub fn verify_password(password: &str, hash: &str) -> Result<bool> {
    Ok(bcrypt::verify(password, hash)?)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_password_hashing() {
        let password = "test_password_123";
        let hash = hash_password(password).unwrap();

        assert!(verify_password(password, &hash).unwrap());
        assert!(!verify_password("wrong_password", &hash).unwrap());
    }
}
