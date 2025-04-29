use crate::auth::models::Claims;
use crate::config::Config;
use bcrypt::{hash, verify, BcryptError, DEFAULT_COST};
use chrono::{Duration, Utc};
use jsonwebtoken::{
    decode, encode, errors::Error as JwtErrorInternal, DecodingKey, EncodingKey, Header, Validation,
};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum JWTError {
    #[error("JWT creation error: {0}")]
    CreationFailed(#[from] JwtErrorInternal),
    #[error("JWT validation error: {0}")]
    ValidationFailed(String),
    #[error("JWT has expired")]
    Expired,
    #[error("Invalid JWT format")]
    InvalidFormat,
}

pub fn hash_password(password: &str) -> Result<String, BcryptError> {
    hash(password, DEFAULT_COST)
}

pub fn verify_password(password: &str, hash: &str) -> Result<bool, BcryptError> {
    verify(password, hash)
}

pub fn create_jwt(user_id: &str, config: &Config) -> Result<String, JWTError> {
    let expiration = Utc::now()
        .checked_add_signed(Duration::hours(config.jwt_expiration_hours as i64))
        .expect("valid timestamp")
        .timestamp();

    let claims = Claims {
        sub: user_id.to_owned(),
        exp: expiration as usize,
    };

    let header = Header::default();
    let key = EncodingKey::from_secret(config.jwt_secret.as_ref());

    encode(&header, &claims, &key).map_err(JWTError::CreationFailed)
}

pub fn validate_jwt(token: &str, config: &Config) -> Result<Claims, JWTError> {
    let key = DecodingKey::from_secret(config.jwt_secret.as_ref());
    let validation = Validation::default();

    decode::<Claims>(token, &key, &validation).map(|data| data.claims)
     .map_err(|err| match err.kind() {
         jsonwebtoken::errors::ErrorKind::ExpiredSignature => JWTError::Expired,
         jsonwebtoken::errors::ErrorKind::InvalidToken => JWTError::InvalidFormat,
         _ => JWTError::ValidationFailed(err.to_string()),
     })
}