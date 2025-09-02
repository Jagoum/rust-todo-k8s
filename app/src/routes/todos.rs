//! # Todo Management Routes
//!
//! Handles authenticated endpoints for CRUD operations on todo items.
//! All routes require valid JWT authentication and operate on user-specific data.

use axum::{extract::{Path, State}, http::StatusCode, response::IntoResponse, Json};
use sqlx::Row;
use uuid::Uuid;
use tracing::{info, warn, error, instrument, debug};

use crate::models::{AppState, AuthedUser, TodoCreateRequest, TodoUpdateRequest};

/// List all todos for the authenticated user
/// 
/// Retrieves all todo items belonging to the authenticated user, ordered by creation date (newest first).
/// Only returns todos owned by the requesting user for security.
/// 
/// # Authentication
/// 
/// Requires valid JWT token in Authorization header.
/// 
/// # Response
/// 
/// - `200 OK` - Array of todo objects with full details
/// - `401 Unauthorized` - Missing or invalid authentication
/// - `500 Internal Server Error` - Database query error
/// 
/// # Response Format
/// 
/// Returns an array of todo objects, each containing:
/// - `id`: Unique identifier (UUID)
/// - `user_id`: Owner's user ID (UUID)
/// - `title`: Todo description
/// - `completed`: Boolean completion status
/// - `created_at`: ISO 8601 formatted creation timestamp
/// 
/// # Examples
/// 
/// ```json
/// GET /todos
/// Authorization: Bearer <jwt_token>
/// 
/// Response: 200 OK
/// [
///   {
///     "id": "550e8400-e29b-41d4-a716-446655440000",
///     "user_id": "6ba7b810-9dad-11d1-80b4-00c04fd430c8",
///     "title": "Complete project documentation",
///     "completed": false,
///     "created_at": "2024-01-15T10:30:00Z"
///   }
/// ]
/// ```
#[instrument(skip(state), fields(user_id = %user.user_id, username = %user.username))]
pub async fn list_todos(State(state): State<AppState>, user: AuthedUser) -> impl IntoResponse {
    info!("Fetching todos for user");
    
    debug!("Querying database for user's todos");
    let rows = sqlx::query(
        r#"SELECT id, user_id, title, completed, to_char(created_at, 'YYYY-MM-DD"T"HH24:MI:SSZ') as created_at FROM todos WHERE user_id = $1 ORDER BY created_at DESC"#
    )
    .bind(user.user_id)
    .fetch_all(&state.pool)
    .await;

    match rows {
        Ok(rows) => {
            let todo_count = rows.len();
            debug!("Found {} todos for user", todo_count);
            
            let todos: Vec<serde_json::Value> = rows.iter().map(|row| {
                serde_json::json!({
                    "id": row.get::<uuid::Uuid, _>("id"),
                    "user_id": row.get::<uuid::Uuid, _>("user_id"),
                    "title": row.get::<String, _>("title"),
                    "completed": row.get::<bool, _>("completed"),
                    "created_at": row.get::<String, _>("created_at")
                })
            }).collect();
            
            info!("Successfully retrieved {} todos", todo_count);
            (StatusCode::OK, Json(todos)).into_response()
        }
        Err(e) => {
            error!("Database error while fetching todos: {}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response()
        }
    }
}

/// Create a new todo item
/// 
/// Creates a new todo item for the authenticated user with the provided title.
/// The todo is initially marked as incomplete and assigned a unique ID.
/// 
/// # Authentication
/// 
/// Requires valid JWT token in Authorization header.
/// 
/// # Request Body
/// 
/// Expects JSON payload with:
/// - `title`: Non-empty string describing the todo item
/// 
/// # Response
/// 
/// - `201 Created` - Todo created successfully with generated ID
/// - `400 Bad Request` - Empty or missing title
/// - `401 Unauthorized` - Missing or invalid authentication
/// - `500 Internal Server Error` - Database insertion error
/// 
/// # Examples
/// 
/// ```json
/// POST /todos
/// Authorization: Bearer <jwt_token>
/// {
///   "title": "Review pull requests"
/// }
/// 
/// Response: 201 Created
/// {
///   "id": "550e8400-e29b-41d4-a716-446655440000"
/// }
/// ```
#[instrument(skip(state, req), fields(user_id = %user.user_id, username = %user.username, title = %req.title))]
pub async fn create_todo(State(state): State<AppState>, user: AuthedUser, Json(req): Json<TodoCreateRequest>) -> impl IntoResponse {
    info!("Creating new todo for user");
    
    if req.title.trim().is_empty() {
        warn!("Todo creation failed: empty title");
        return (StatusCode::BAD_REQUEST, "title required").into_response();
    }

    let id = Uuid::new_v4();
    debug!("Generated todo ID: {}", id);
    
    debug!("Inserting todo into database");
    let res = sqlx::query(
        "INSERT INTO todos (id, user_id, title) VALUES ($1, $2, $3)"
    )
    .bind(id)
    .bind(user.user_id)
    .bind(req.title)
    .execute(&state.pool)
    .await;

    match res {
        Ok(_) => {
            info!("Todo created successfully");
            (StatusCode::CREATED, Json(serde_json::json!({"id": id}))).into_response()
        },
        Err(e) => {
            error!("Database error while creating todo: {}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response()
        }
    }
}

/// Update an existing todo item
/// 
/// Updates specific fields of a todo item owned by the authenticated user.
/// Supports partial updates - only provided fields are modified.
/// 
/// # Authentication
/// 
/// Requires valid JWT token in Authorization header.
/// 
/// # Path Parameters
/// 
/// - `id`: UUID of the todo item to update
/// 
/// # Request Body
/// 
/// Expects JSON payload with optional fields:
/// - `title`: New title for the todo item
/// - `completed`: New completion status (true/false)
/// 
/// # Response
/// 
/// - `200 OK` - Todo updated successfully
/// - `400 Bad Request` - No fields provided for update
/// - `401 Unauthorized` - Missing or invalid authentication
/// - `404 Not Found` - Todo not found or not owned by user
/// - `500 Internal Server Error` - Database update error
/// 
/// # Authorization
/// 
/// Users can only update their own todo items. Attempting to update
/// another user's todo will return 404 Not Found.
/// 
/// # Examples
/// 
/// ```json
/// PUT /todos/550e8400-e29b-41d4-a716-446655440000
/// Authorization: Bearer <jwt_token>
/// {
///   "completed": true
/// }
/// 
/// Response: 200 OK
/// {
///   "updated": true
/// }
/// ```
#[instrument(skip(state, req), fields(user_id = %user.user_id, username = %user.username, todo_id = %id))]
pub async fn update_todo(State(state): State<AppState>, user: AuthedUser, Path(id): Path<Uuid>, Json(req): Json<TodoUpdateRequest>) -> impl IntoResponse {
    info!("Updating todo for user");
    
    let mut title = None;
    let mut completed = None;
    if let Some(t) = req.title { 
        debug!("Updating title to: {}", t);
        title = Some(t); 
    }
    if let Some(c) = req.completed { 
        debug!("Updating completed status to: {}", c);
        completed = Some(c); 
    }

    if title.is_none() && completed.is_none() {
        warn!("Todo update failed: no fields to update");
        return (StatusCode::BAD_REQUEST, "nothing to update").into_response();
    }

    debug!("Executing update query");
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
            let rows_affected = done.rows_affected();
            if rows_affected == 0 { 
                warn!("Todo update failed: todo not found or not owned by user");
                return (StatusCode::NOT_FOUND, "not found").into_response(); 
            }
            info!("Todo updated successfully");
            (StatusCode::OK, Json(serde_json::json!({"updated": true}))).into_response()
        }
        Err(e) => {
            error!("Database error while updating todo: {}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response()
        }
    }
}

/// Delete a todo item
/// 
/// Permanently removes a todo item owned by the authenticated user.
/// This operation cannot be undone.
/// 
/// # Authentication
/// 
/// Requires valid JWT token in Authorization header.
/// 
/// # Path Parameters
/// 
/// - `id`: UUID of the todo item to delete
/// 
/// # Response
/// 
/// - `200 OK` - Todo deleted successfully
/// - `401 Unauthorized` - Missing or invalid authentication
/// - `404 Not Found` - Todo not found or not owned by user
/// - `500 Internal Server Error` - Database deletion error
/// 
/// # Authorization
/// 
/// Users can only delete their own todo items. Attempting to delete
/// another user's todo will return 404 Not Found.
/// 
/// # Examples
/// 
/// ```json
/// DELETE /todos/550e8400-e29b-41d4-a716-446655440000
/// Authorization: Bearer <jwt_token>
/// 
/// Response: 200 OK
/// {
///   "deleted": true
/// }
/// ```
#[instrument(skip(state), fields(user_id = %user.user_id, username = %user.username, todo_id = %id))]
pub async fn delete_todo(State(state): State<AppState>, user: AuthedUser, Path(id): Path<Uuid>) -> impl IntoResponse {
    info!("Deleting todo for user");
    
    debug!("Executing delete query");
    let res = sqlx::query(
        "DELETE FROM todos WHERE id = $1 AND user_id = $2"
    )
    .bind(id)
    .bind(user.user_id)
    .execute(&state.pool)
    .await;

    match res {
        Ok(done) => {
            let rows_affected = done.rows_affected();
            if rows_affected == 0 { 
                warn!("Todo deletion failed: todo not found or not owned by user");
                return (StatusCode::NOT_FOUND, "not found").into_response(); 
            }
            info!("Todo deleted successfully");
            (StatusCode::OK, Json(serde_json::json!({"deleted": true}))).into_response()
        }
        Err(e) => {
            error!("Database error while deleting todo: {}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response()
        }
    }
}