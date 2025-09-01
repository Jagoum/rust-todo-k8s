use std::time::SystemTime;

use axum::{
    body::Body,
    extract::{FromRequestParts, State},
    http::{request::Parts, Request, StatusCode},
    middleware::Next,
    response::Response,
};
use jsonwebtoken as jwt;
use argon2::{Argon2, password_hash::{PasswordHash, PasswordHasher, PasswordVerifier, SaltString}};
use rand::rngs::OsRng;
use uuid::Uuid;

use crate::models::{AppState, AuthedUser, Claims};

pub async fn auth_middleware(
    State(state): State<AppState>,
    mut req: Request<Body>,
    next: Next,
) -> Result<Response, (StatusCode, String)> {
    let token = extract_bearer(req.headers()).ok_or((StatusCode::UNAUTHORIZED, "missing bearer".to_string()))?;

    let decoding_key = jwt::DecodingKey::from_secret(state.jwt_secret.as_bytes());
    let validation = jwt::Validation::new(jwt::Algorithm::HS256);
    let data = jwt::decode::<Claims>(&token, &decoding_key, &validation)
        .map_err(|_| (StatusCode::UNAUTHORIZED, "token invalid".to_string()))?;

    let user_id = Uuid::parse_str(&data.claims.sub)
        .map_err(|_| (StatusCode::UNAUTHORIZED, "bad sub".to_string()))?;

    req.extensions_mut().insert(AuthedUser { user_id, username: data.claims.username.clone() });
    Ok(next.run(req).await)
}

fn extract_bearer(headers: &axum::http::HeaderMap) -> Option<String> {
    let value = headers.get(axum::http::header::AUTHORIZATION)?;
    let s = value.to_str().ok()?;
    let prefix = "Bearer ";
    if s.starts_with(prefix) { Some(s[prefix.len()..].to_string()) } else { None }
}

pub fn issue_token(secret: &str, user_id: Uuid, username: &str) -> anyhow::Result<String> {
    let now = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH)?.as_secs();
    let exp = now + 60 * 60 * 24; // 24 hours
    let claims = Claims { sub: user_id.to_string(), username: username.to_string(), exp };
    let key = jwt::EncodingKey::from_secret(secret.as_bytes());
    let token = jwt::encode(&jwt::Header::new(jwt::Algorithm::HS256), &claims, &key)?;
    Ok(token)
}

pub fn hash_password(password: &str) -> anyhow::Result<String> {
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();
    let hash = argon2.hash_password(password.as_bytes(), &salt)?.to_string();
    Ok(hash)
}

pub fn verify_password(password: &str, password_hash: &str) -> anyhow::Result<bool> {
    let parsed_hash = PasswordHash::new(password_hash)?;
    Ok(Argon2::default().verify_password(password.as_bytes(), &parsed_hash).is_ok())
}

#[axum::async_trait]
impl<S> FromRequestParts<S> for AuthedUser
where
    S: Send + Sync,
{
    type Rejection = (axum::http::StatusCode, String);

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        parts.extensions.get::<AuthedUser>()
            .cloned()
            .ok_or((axum::http::StatusCode::UNAUTHORIZED, "no user in request".to_string()))
    }
}