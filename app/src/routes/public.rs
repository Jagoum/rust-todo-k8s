//! # Public API Routes
//!
//! Handles unauthenticated endpoints for user registration and login.
//! These routes are accessible without JWT tokens and provide authentication functionality.

use axum::{extract::State, http::StatusCode, response::IntoResponse, Json};
use sqlx::Row;
use uuid::Uuid;
use tracing::{info, warn, error, instrument, debug};

use crate::{
    auth,
    models::{AppState, LoginRequest, LoginResponse, RegisterRequest},
};

/// Register a new user account
/// 
/// Creates a new user account with username and password authentication.
/// Validates input requirements and ensures username uniqueness.
/// 
/// # Request Body
/// 
/// Expects JSON payload with:
/// - `username`: Non-empty string identifier (must be unique)
/// - `password`: String with minimum 6 characters
/// 
/// # Response
/// 
/// - `201 Created` - User created successfully with user ID
/// - `400 Bad Request` - Invalid input (empty username or short password)
/// - `409 Conflict` - Username already exists
/// - `500 Internal Server Error` - Database or hashing error
/// 
/// # Security
/// 
/// - Passwords are hashed using Argon2 before storage
/// - Usernames are checked for uniqueness via database constraint
/// - Input validation prevents empty usernames and weak passwords
/// 
/// # Examples
/// 
/// ```json
/// POST /register
/// {
///   "username": "john_doe",
///   "password": "secure_password123"
/// }
/// 
/// Response: 201 Created
/// {
///   "id": "550e8400-e29b-41d4-a716-446655440000"
/// }
/// ```
#[instrument(skip(state, payload), fields(username = %payload.username))]
pub async fn register(State(state): State<AppState>, Json(payload): Json<RegisterRequest>) -> impl IntoResponse {
    info!("User registration attempt");
    
    if payload.username.trim().is_empty() || payload.password.len() < 6 {
        warn!("Invalid registration payload: empty username or short password");
        return (StatusCode::BAD_REQUEST, "invalid payload").into_response();
    }

    let user_id = Uuid::new_v4();
    debug!("Generated user ID: {}", user_id);
    
    let password_hash = match auth::hash_password(&payload.password) {
        Ok(h) => {
            debug!("Password hashed successfully");
            h
        },
        Err(e) => {
            error!("Password hashing failed: {}", e);
            return (StatusCode::INTERNAL_SERVER_ERROR, format!("hash error: {}", e)).into_response();
        }
    };

    debug!("Inserting user into database");
    let res = sqlx::query(
        "INSERT INTO users (id, username, password_hash) VALUES ($1, $2, $3)"
    )
    .bind(user_id)
    .bind(&payload.username)
    .bind(password_hash)
    .execute(&state.pool)
    .await;

    match res {
        Ok(_) => {
            info!("User registered successfully");
            (StatusCode::CREATED, Json(serde_json::json!({"id": user_id}))).into_response()
        },
        Err(e) => {
            if let Some(db_err) = e.as_database_error() {
                if db_err.code().as_deref() == Some("23505") {
                    warn!("Registration failed: username already taken");
                    return (StatusCode::CONFLICT, "username taken").into_response();
                } else {
                    // Log the specific database error code and message for better debugging
                    error!("Database error during registration: Code: {:?}, Message: {:?}", db_err.code(), db_err.message());
                    return (StatusCode::INTERNAL_SERVER_ERROR, format!("db error: {:?}", db_err)).into_response();
                }
            } else {
                // Log if the error is not a database error
                error!("Non-database error during registration: {}", e);
                return (StatusCode::INTERNAL_SERVER_ERROR, format!("unexpected error: {}", e)).into_response();
            }
        }
    }
}

/// Authenticate user and issue JWT token
/// 
/// Validates user credentials and returns a JWT token for authenticated requests.
/// Token is valid for 24 hours and should be included in Authorization header.
/// 
/// # Request Body
/// 
/// Expects JSON payload with:
/// - `username`: User's registered username
/// - `password`: User's plain text password
/// 
/// # Response
/// 
/// - `200 OK` - Authentication successful with JWT token
/// - `401 Unauthorized` - Invalid username or password
/// - `500 Internal Server Error` - Database or token generation error
/// 
/// # Security
/// 
/// - Uses constant-time password verification to prevent timing attacks
/// - Returns generic "invalid credentials" message for both missing users and wrong passwords
/// - JWT tokens are signed with server secret and include expiration
/// 
/// # Token Usage
/// 
/// Include the returned token in subsequent requests:
/// ```
/// Authorization: Bearer <token>
/// ```
/// 
/// # Examples
/// 
/// ```json
/// POST /login
/// {
///   "username": "john_doe",
///   "password": "secure_password123"
/// }
/// 
/// Response: 200 OK
/// {
///   "token": "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9..."
/// }
/// ```
#[instrument(skip(state, payload), fields(username = %payload.username))]
pub async fn login(State(state): State<AppState>, Json(payload): Json<LoginRequest>) -> impl IntoResponse {
    info!("User login attempt");
    
    debug!("Querying database for user");
    let row = sqlx::query("SELECT id, username, password_hash FROM users WHERE username = $1")
    .bind(&payload.username)
    .fetch_optional(&state.pool)
    .await;

    let row = match row {
        Ok(Some(r)) => {
            debug!("User found in database");
            r
        },
        Ok(None) => {
            warn!("Login failed: user not found");
            return (StatusCode::UNAUTHORIZED, "invalid credentials").into_response();
        }
        Err(e) => {
            error!("Database error during login: {}", e);
            return (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response();
        }
    };

    let password_hash: String = match row.try_get("password_hash") {
        Ok(h) => h,
        Err(e) => {
            error!("Failed to extract password hash from database row: {}", e);
            return (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response();
        }
    };
    let user_id: uuid::Uuid = match row.try_get("id") {
        Ok(id) => id,
        Err(e) => {
            error!("Failed to extract user ID from database row: {}", e);
            return (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response();
        }
    };
    let username: String = match row.try_get("username") {
        Ok(u) => u,
        Err(e) => {
            error!("Failed to extract username from database row: {}", e);
            return (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response();
        }
    };
    
    debug!("Verifying password");
    if auth::verify_password(&payload.password, &password_hash).unwrap_or(false) {
        info!("Login successful, generating JWT token");
        let token = match auth::issue_token(&state.jwt_secret, user_id, &username) {
            Ok(t) => t,
            Err(e) => {
                error!("Failed to generate JWT token: {}", e);
                return (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response();
            }
        };
        return (StatusCode::OK, Json(LoginResponse { token })).into_response();
    }

    warn!("Login failed: invalid password");
    (StatusCode::UNAUTHORIZED, "invalid credentials").into_response()
}