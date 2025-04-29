use crate::{
    auth::utils::{JWTError, validate_jwt},
    state::AppState,
};
use axum::{
    extract::{FromRef, FromRequestParts, Request, State},
    http::{HeaderMap, StatusCode, request::Parts},
    middleware::Next,
    response::Response,
};
use tracing::warn;

use super::models::UserId;

const AUTH_HEADER_NAME: &str = "Authorization";
const AUTH_SCHEME: &str = "Bearer ";

fn extract_token(headers: &HeaderMap) -> Option<String> {
    headers
        .get(AUTH_HEADER_NAME)
        .and_then(|header| header.to_str().ok())
        .and_then(|value| value.strip_prefix(AUTH_SCHEME))
        .map(|token| token.to_string())
}

pub async fn auth_middleware(
    State(state): State<AppState>,
    mut req: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    let headers = req.headers();
    let token = extract_token(headers).ok_or_else(|| {
        warn!("Authentication failed: Missing or malformed Authorization header");
        StatusCode::UNAUTHORIZED
    })?;

    let claims = validate_jwt(&token, &state.config).map_err(|e| match e {
        JWTError::Expired => StatusCode::UNAUTHORIZED,
        JWTError::ValidationFailed(_) | JWTError::InvalidFormat => StatusCode::UNAUTHORIZED,
        JWTError::CreationFailed(_) => StatusCode::INTERNAL_SERVER_ERROR,
    })?;

    let user_id = UserId(claims.sub);
    req.extensions_mut().insert(user_id);

    Ok(next.run(req).await)
}

impl<S> FromRequestParts<S> for UserId
where
    AppState: FromRef<S>,
    S: Send + Sync,
{
    type Rejection = (StatusCode, &'static str);

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        parts.extensions.get::<UserId>().cloned().ok_or((
            StatusCode::INTERNAL_SERVER_ERROR,
            "User ID not found in request extensions.",
        ))
    }
}
