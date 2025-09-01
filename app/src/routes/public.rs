use axum::{extract::State, http::StatusCode, response::IntoResponse, Json};
use sqlx::Row;
use uuid::Uuid;

use crate::{
    auth,
    models::{AppState, LoginRequest, LoginResponse, RegisterRequest},
};

pub async fn register(State(state): State<AppState>, Json(payload): Json<RegisterRequest>) -> impl IntoResponse {
    if payload.username.trim().is_empty() || payload.password.len() < 6 {
        return (StatusCode::BAD_REQUEST, "invalid payload").into_response();
    }

    let user_id = Uuid::new_v4();
    let password_hash = match auth::hash_password(&payload.password) {
        Ok(h) => h,
        Err(e) => return (StatusCode::INTERNAL_SERVER_ERROR, format!("hash error: {}", e)).into_response(),
    };

    let res = sqlx::query(
        "INSERT INTO users (id, username, password_hash) VALUES ($1, $2, $3)"
    )
    .bind(user_id)
    .bind(&payload.username)
    .bind(password_hash)
    .execute(&state.pool)
    .await;

    match res {
        Ok(_) => (StatusCode::CREATED, Json(serde_json::json!({"id": user_id}))).into_response(),
        Err(e) => {
            if let Some(db_err) = e.as_database_error() {
                if db_err.code().as_deref() == Some("23505") {
                    return (StatusCode::CONFLICT, "username taken").into_response();
                }
            }
            (StatusCode::INTERNAL_SERVER_ERROR, format!("db error: {}", e)).into_response()
        }
    }
}

pub async fn login(State(state): State<AppState>, Json(payload): Json<LoginRequest>) -> impl IntoResponse {
    let row = sqlx::query("SELECT id, username, password_hash FROM users WHERE username = $1")
    .bind(&payload.username)
    .fetch_optional(&state.pool)
    .await;

    let row = match row {
        Ok(Some(r)) => r,
        Ok(None) => {
            return (StatusCode::UNAUTHORIZED, "invalid credentials").into_response();
        }
        Err(e) => {
            return (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response();
        }
    };

    let password_hash: String = match row.try_get("password_hash") {
        Ok(h) => h,
        Err(e) => return (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
    };
    let user_id: uuid::Uuid = match row.try_get("id") {
        Ok(id) => id,
        Err(e) => return (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
    };
    let username: String = match row.try_get("username") {
        Ok(u) => u,
        Err(e) => return (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
    };
    
    if auth::verify_password(&payload.password, &password_hash).unwrap_or(false) {
        let token = match auth::issue_token(&state.jwt_secret, user_id, &username) {
            Ok(t) => t,
            Err(e) => return (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
        };
        return (StatusCode::OK, Json(LoginResponse { token })).into_response();
    }

    (StatusCode::UNAUTHORIZED, "invalid credentials").into_response()
}