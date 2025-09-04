//! # Authentication and Authorization
//!
//! Handles JWT-based authentication, password hashing, and authorization middleware.
//! Provides secure user authentication using industry-standard practices.

use std::time::SystemTime;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

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
use serde::{Deserialize, Serialize};
use tracing::{info, warn, error, instrument, debug};

use crate::models::{AppState, AuthedUser, Claims};

/// JWKS (JSON Web Key Set) structure for Keycloak
#[derive(Debug, Serialize, Deserialize)]
struct Jwks {
    keys: Vec<Jwk>,
}

#[derive(Debug, Serialize, Deserialize)]
struct Jwk {
    kid: String,
    kty: String,
    n: String,
    e: String,
}

/// Keycloak configuration
#[derive(Clone)]
pub struct KeycloakConfig {
    pub issuer_url: String,
    pub client_id: String,
    pub jwks_url: String,
}

impl KeycloakConfig {
    pub fn from_env() -> Option<Self> {
        let issuer_url = std::env::var("KEYCLOAK_ISSUER_URL").ok()?;
        let client_id = std::env::var("KEYCLOAK_CLIENT_ID").ok()?;
        let jwks_url = format!("{}/.well-known/jwks", issuer_url);

        Some(Self {
            issuer_url,
            client_id,
            jwks_url,
        })
    }
}

/// Cached JWKS keys
type JwksCache = Arc<RwLock<HashMap<String, jwt::DecodingKey>>>;

/// Authentication middleware for protected routes
/// 
/// Validates JWT tokens from the Authorization header and extracts user information.
/// Adds authenticated user data to request extensions for use in route handlers.
/// 
/// # Arguments
/// 
/// * `state` - Application state containing JWT secret
/// * `req` - HTTP request to authenticate
/// * `next` - Next middleware/handler in the chain
/// 
/// # Returns
/// 
/// Returns the response from the next handler if authentication succeeds,
/// or an error response if authentication fails.
/// 
/// # Errors
/// 
/// Returns HTTP error responses for:
/// - `401 Unauthorized` - Missing or invalid Authorization header
/// - `401 Unauthorized` - Invalid or expired JWT token
/// - `401 Unauthorized` - Malformed user ID in token claims
/// 
/// # Examples
/// 
/// ```rust
/// // Applied as middleware to protected routes
/// let protected = Router::new()
///     .route("/todos", get(list_todos))
///     .layer(from_fn_with_state(state, auth_middleware));
/// ```
#[instrument(skip(state, req, next), fields(user_id, username))]
pub async fn auth_middleware(
    State(state): State<AppState>,
    mut req: Request<Body>,
    next: Next,
) -> Result<Response, (StatusCode, String)> {
    debug!("Processing authentication middleware");

    let token = extract_bearer(req.headers()).ok_or_else(|| {
        warn!("Missing Authorization header");
        (StatusCode::UNAUTHORIZED, "missing bearer".to_string())
    })?;

    // Try local JWT validation first
    let decoding_key = jwt::DecodingKey::from_secret(state.jwt_secret.as_bytes());
    let validation = jwt::Validation::new(jwt::Algorithm::HS256);
    let local_result = jwt::decode::<Claims>(&token, &decoding_key, &validation);

    if let Ok(data) = local_result {
        let user_id = Uuid::parse_str(&data.claims.sub).map_err(|e| {
            error!("Invalid user ID in token claims: {}", e);
            (StatusCode::UNAUTHORIZED, "bad sub".to_string())
        })?;

        tracing::Span::current()
            .record("user_id", &tracing::field::display(&user_id))
            .record("username", &data.claims.username);

        info!("User authenticated successfully with local JWT");
        req.extensions_mut().insert(AuthedUser { user_id, username: data.claims.username.clone() });
        return Ok(next.run(req).await);
    }

    // If local validation fails, try Keycloak validation
    let keycloak_config = match KeycloakConfig::from_env() {
        Some(cfg) => cfg,
        None => {
            warn!("Keycloak config missing");
            return Err((StatusCode::UNAUTHORIZED, "invalid token".to_string()));
        }
    };

    // Fetch JWKS keys cache (in-memory)
    static JWKS_CACHE: once_cell::sync::Lazy<JwksCache> = once_cell::sync::Lazy::new(|| Arc::new(RwLock::new(HashMap::new())));
    let jwks_cache = JWKS_CACHE.clone();

    // Decode token header to get kid
    let header = jwt::decode_header(&token).map_err(|e| {
        warn!("Failed to decode JWT header: {}", e);
        (StatusCode::UNAUTHORIZED, "invalid token".to_string())
    })?;

    let kid = header.kid.ok_or_else(|| {
        warn!("JWT token missing kid");
        (StatusCode::UNAUTHORIZED, "invalid token".to_string())
    })?;

    // Try to get decoding key from cache
    let key_opt = {
        let cache = jwks_cache.read().await;
        cache.get(&kid).cloned()
    };

    let decoding_key = if let Some(key) = key_opt {
        key
    } else {
        // Fetch JWKS from Keycloak
        let jwks_resp = reqwest::get(&keycloak_config.jwks_url).await.map_err(|e| {
            error!("Failed to fetch JWKS: {}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, "jwks fetch error".to_string())
        })?;

        let jwks: Jwks = jwks_resp.json().await.map_err(|e| {
            error!("Failed to parse JWKS JSON: {}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, "jwks parse error".to_string())
        })?;

        // Find key by kid
        let jwk = jwks.keys.into_iter().find(|k| k.kid == kid).ok_or_else(|| {
            warn!("JWKS key not found for kid: {}", kid);
            (StatusCode::UNAUTHORIZED, "invalid token".to_string())
        })?;

        // Construct decoding key
        let key = jwt::DecodingKey::from_rsa_components(&jwk.n, &jwk.e).map_err(|e| {
            error!("Failed to create decoding key: {}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, "key error".to_string())
        })?;

        // Cache the key
        {
            let mut cache = jwks_cache.write().await;
            cache.insert(kid.clone(), key.clone());
        }

        key
    };

    // Validate token with Keycloak key
    let mut validation = jwt::Validation::new(jwt::Algorithm::RS256);
    validation.set_audience(&[keycloak_config.client_id.as_str()]);
    validation.set_issuer(&[keycloak_config.issuer_url.as_str()]);

    let data = jwt::decode::<Claims>(&token, &decoding_key, &validation).map_err(|e| {
        warn!("Invalid Keycloak JWT token: {}", e);
        (StatusCode::UNAUTHORIZED, "token invalid".to_string())
    })?;

    let user_id = Uuid::parse_str(&data.claims.sub).map_err(|e| {
        error!("Invalid user ID in token claims: {}", e);
        (StatusCode::UNAUTHORIZED, "bad sub".to_string())
    })?;

    tracing::Span::current()
        .record("user_id", &tracing::field::display(&user_id))
        .record("username", &data.claims.username);

    info!("User authenticated successfully with Keycloak JWT");
    req.extensions_mut().insert(AuthedUser { user_id, username: data.claims.username.clone() });
    Ok(next.run(req).await)
}

/// Extract Bearer token from Authorization header
/// 
/// Parses the Authorization header to extract the JWT token.
/// Expects the format: "Bearer <token>"
/// 
/// # Arguments
/// 
/// * `headers` - HTTP headers from the request
/// 
/// # Returns
/// 
/// Returns `Some(token)` if a valid Bearer token is found, `None` otherwise.
/// 
/// # Examples
/// 
/// ```rust
/// let headers = /* request headers */;
/// if let Some(token) = extract_bearer(&headers) {
///     // Process token
/// }
/// ```
fn extract_bearer(headers: &axum::http::HeaderMap) -> Option<String> {
    let value = headers.get(axum::http::header::AUTHORIZATION)?;
    let s = value.to_str().ok()?;
    let prefix = "Bearer ";
    if s.starts_with(prefix) { Some(s[prefix.len()..].to_string()) } else { None }
}

/// Generate a JWT token for authenticated user
/// 
/// Creates a signed JWT token containing user identification and expiration.
/// Token is valid for 24 hours from issuance.
/// 
/// # Arguments
/// 
/// * `secret` - Secret key for token signing
/// * `user_id` - Unique identifier of the user
/// * `username` - Username for display purposes
/// 
/// # Returns
/// 
/// Returns a signed JWT token string on success.
/// 
/// # Errors
/// 
/// Returns an error if:
/// - System time cannot be determined
/// - Token encoding fails
/// 
/// # Examples
/// 
/// ```rust
/// let token = issue_token("secret", user_id, "john_doe")?;
/// // Token can be used in Authorization: Bearer <token>
/// ```
#[instrument(skip(secret), fields(user_id = %user_id, username = %username))]
pub fn issue_token(secret: &str, user_id: Uuid, username: &str) -> anyhow::Result<String> {
    debug!("Generating JWT token for user");
    
    let now = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH)?.as_secs();
    let exp = now + 60 * 60 * 24; // 24 hours
    let claims = Claims {
        sub: user_id.to_string(),
        username: username.to_string(),
        exp,
        iss: None,
        aud: None,
    };
    let key = jwt::EncodingKey::from_secret(secret.as_bytes());
    let token = jwt::encode(&jwt::Header::new(jwt::Algorithm::HS256), &claims, &key)?;
    
    info!("JWT token generated successfully, expires in 24 hours");
    Ok(token)
}

/// Hash a password using Argon2
/// 
/// Securely hashes a plain text password using the Argon2 algorithm with a random salt.
/// Uses cryptographically secure random number generation for salt creation.
/// 
/// # Arguments
/// 
/// * `password` - Plain text password to hash
/// 
/// # Returns
/// 
/// Returns the hashed password as a string suitable for database storage.
/// 
/// # Errors
/// 
/// Returns an error if:
/// - Random salt generation fails
/// - Password hashing fails
/// 
/// # Security
/// 
/// - Uses Argon2 with default parameters (recommended for security)
/// - Generates a unique salt for each password
/// - Resistant to rainbow table and timing attacks
/// 
/// # Examples
/// 
/// ```rust
/// let hashed = hash_password("user_password")?;
/// // Store hashed password in database
/// ```
pub fn hash_password(password: &str) -> anyhow::Result<String> {
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();
    let hash = argon2.hash_password(password.as_bytes(), &salt)?.to_string();
    Ok(hash)
}

/// Verify a password against its hash
/// 
/// Verifies that a plain text password matches a previously hashed password.
/// Uses constant-time comparison to prevent timing attacks.
/// 
/// # Arguments
/// 
/// * `password` - Plain text password to verify
/// * `password_hash` - Previously hashed password from database
/// 
/// # Returns
/// 
/// Returns `Ok(true)` if the password matches, `Ok(false)` if it doesn't.
/// 
/// # Errors
/// 
/// Returns an error if:
/// - Password hash format is invalid
/// - Verification process fails
/// 
/// # Security
/// 
/// - Uses constant-time comparison to prevent timing attacks
/// - Handles malformed hashes gracefully
/// - Does not leak information about hash validity
/// 
/// # Examples
/// 
/// ```rust
/// let is_valid = verify_password("user_password", &stored_hash)?;
/// if is_valid {
///     // Password is correct
/// }
/// ```
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