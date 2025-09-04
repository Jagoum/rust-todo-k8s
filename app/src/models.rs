//! # Data Models and DTOs
//!
//! Contains all data structures used throughout the application including:
//! - Application state management
//! - JWT claims and authentication models
//! - HTTP request/response DTOs
//! - Database entity models

use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use uuid::Uuid;

/// Application state shared across all request handlers
/// 
/// Contains shared resources that need to be accessible throughout the application
/// lifecycle, including database connections and configuration.
#[derive(Clone)]
pub struct AppState {
    /// PostgreSQL connection pool for database operations
    pub pool: PgPool,
    /// Secret key used for JWT token signing and verification
    pub jwt_secret: String,
}

/// JWT token claims structure
///
/// Contains the payload data embedded in JWT tokens for user authentication.
/// Supports both local JWT and Keycloak JWT formats.
#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    /// Subject - User ID as string (standard JWT claim)
    pub sub: String,
    /// Username for display purposes (local JWT) or preferred_username (Keycloak)
    #[serde(alias = "preferred_username")]
    pub username: String,
    /// Expiration timestamp as Unix epoch seconds (standard JWT claim)
    pub exp: u64,
    /// Issuer (Keycloak realm URL) - optional for local JWT
    #[serde(default)]
    pub iss: Option<String>,
    /// Audience (client ID) - optional for local JWT
    #[serde(default)]
    pub aud: Option<String>,
}

/// User registration request payload
/// 
/// Data transfer object for user registration endpoint.
/// Contains the minimum required information to create a new user account.
#[derive(Debug, Serialize, Deserialize)]
pub struct RegisterRequest {
    /// Desired username (must be unique)
    pub username: String,
    /// Plain text password (will be hashed before storage)
    pub password: String,
}

/// User login request payload
/// 
/// Data transfer object for user authentication endpoint.
/// Contains credentials for user login verification.
#[derive(Debug, Serialize, Deserialize)]
pub struct LoginRequest {
    /// Username for authentication
    pub username: String,
    /// Plain text password for verification
    pub password: String,
}

/// User login response payload
/// 
/// Data transfer object returned after successful authentication.
/// Contains the JWT token for subsequent authenticated requests.
#[derive(Debug, Serialize, Deserialize)]
pub struct LoginResponse {
    /// JWT token for authentication (include in Authorization header)
    pub token: String,
}

/// Todo item creation request payload
/// 
/// Data transfer object for creating new todo items.
/// Contains the minimum required information for a new todo.
#[derive(Debug, Serialize, Deserialize)]
pub struct TodoCreateRequest {
    /// Title/description of the todo item
    pub title: String,
}

/// Todo item update request payload
/// 
/// Data transfer object for updating existing todo items.
/// All fields are optional to support partial updates.
#[derive(Debug, Serialize, Deserialize)]
pub struct TodoUpdateRequest {
    /// New title for the todo item (optional)
    pub title: Option<String>,
    /// New completion status (optional)
    pub completed: Option<bool>,
}

/// Todo item database entity and response model
/// 
/// Represents a todo item as stored in the database and returned in API responses.
/// Contains all fields including metadata like creation timestamp.
#[derive(Debug, Serialize, Deserialize)]
pub struct Todo {
    /// Unique identifier for the todo item
    pub id: Uuid,
    /// ID of the user who owns this todo item
    pub user_id: Uuid,
    /// Title/description of the todo item
    pub title: String,
    /// Whether the todo item has been completed
    pub completed: bool,
    /// ISO 8601 formatted creation timestamp
    pub created_at: String,
}

/// Authenticated user information extracted from JWT token
/// 
/// Contains user information extracted from a valid JWT token during authentication.
/// Used throughout the application to identify the current user for authorization.
#[derive(Debug, Clone)]
pub struct AuthedUser {
    /// Unique identifier of the authenticated user
    pub user_id: Uuid,
    /// Username of the authenticated user (for display/logging purposes)
    #[allow(dead_code)]
    pub username: String,
}