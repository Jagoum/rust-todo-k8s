use axum::{extract::{Path, State}, http::StatusCode, response::IntoResponse, Json};
use sqlx::Row;
use uuid::Uuid;

use crate::models::{AppState, AuthedUser, TodoCreateRequest, TodoUpdateRequest};

pub async fn list_todos(State(state): State<AppState>, user: AuthedUser) -> impl IntoResponse {
    let rows = sqlx::query(
        r#"SELECT id, user_id, title, completed, to_char(created_at, 'YYYY-MM-DD"T"HH24:MI:SSZ') as created_at FROM todos WHERE user_id = $1 ORDER BY created_at DESC"#
    )
    .bind(user.user_id)
    .fetch_all(&state.pool)
    .await;

    match rows {
        Ok(rows) => {
            let todos: Vec<serde_json::Value> = rows.iter().map(|row| {
                serde_json::json!({
                    "id": row.get::<uuid::Uuid, _>("id"),
                    "user_id": row.get::<uuid::Uuid, _>("user_id"),
                    "title": row.get::<String, _>("title"),
                    "completed": row.get::<bool, _>("completed"),
                    "created_at": row.get::<String, _>("created_at")
                })
            }).collect();
            (StatusCode::OK, Json(todos)).into_response()
        }
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
    }
}

pub async fn create_todo(State(state): State<AppState>, user: AuthedUser, Json(req): Json<TodoCreateRequest>) -> impl IntoResponse {
    if req.title.trim().is_empty() {
        return (StatusCode::BAD_REQUEST, "title required").into_response();
    }

    let id = Uuid::new_v4();
    let res = sqlx::query(
        "INSERT INTO todos (id, user_id, title) VALUES ($1, $2, $3)"
    )
    .bind(id)
    .bind(user.user_id)
    .bind(req.title)
    .execute(&state.pool)
    .await;

    match res {
        Ok(_) => (StatusCode::CREATED, Json(serde_json::json!({"id": id}))).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
    }
}

pub async fn update_todo(State(state): State<AppState>, user: AuthedUser, Path(id): Path<Uuid>, Json(req): Json<TodoUpdateRequest>) -> impl IntoResponse {
    let mut title = None;
    let mut completed = None;
    if let Some(t) = req.title { title = Some(t); }
    if let Some(c) = req.completed { completed = Some(c); }

    if title.is_none() && completed.is_none() {
        return (StatusCode::BAD_REQUEST, "nothing to update").into_response();
    }

    let res = sqlx::query(
        r#"
        UPDATE todos SET
          title = COALESCE($1, title),
          completed = COALESCE($2, completed)
        WHERE id = $3 AND user_id = $4
        "#
    )
    .bind(title)
    .bind(completed)
    .bind(id)
    .bind(user.user_id)
    .execute(&state.pool)
    .await;

    match res {
        Ok(done) => {
            if done.rows_affected() == 0 { return (StatusCode::NOT_FOUND, "not found").into_response(); }
            (StatusCode::OK, Json(serde_json::json!({"updated": true}))).into_response()
        }
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
    }
}

pub async fn delete_todo(State(state): State<AppState>, user: AuthedUser, Path(id): Path<Uuid>) -> impl IntoResponse {
    let res = sqlx::query(
        "DELETE FROM todos WHERE id = $1 AND user_id = $2"
    )
    .bind(id)
    .bind(user.user_id)
    .execute(&state.pool)
    .await;

    match res {
        Ok(done) => {
            if done.rows_affected() == 0 { return (StatusCode::NOT_FOUND, "not found").into_response(); }
            (StatusCode::OK, Json(serde_json::json!({"deleted": true}))).into_response()
        }
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
    }
}