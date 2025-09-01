use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use uuid::Uuid;

#[derive(Clone)]
pub struct AppState {
    pub pool: PgPool,
    pub jwt_secret: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,
    pub username: String,
    pub exp: u64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RegisterRequest {
    pub username: String,
    pub password: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LoginResponse {
    pub token: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TodoCreateRequest {
    pub title: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TodoUpdateRequest {
    pub title: Option<String>,
    pub completed: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Todo {
    pub id: Uuid,
    pub user_id: Uuid,
    pub title: String,
    pub completed: bool,
    pub created_at: String,
}
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct AuthedUser {
    pub user_id: Uuid,
    pub username: String,
}