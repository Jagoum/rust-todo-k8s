use actix_web::{web, HttpRequest, HttpResponse, Result};
use sqlx::PgPool;
use uuid::Uuid;
use validator::Validate;

use crate::middleware::auth::get_user_id_from_request;
use crate::models::{ApiResponse, UpdateUserRequest, UserResponse};

pub async fn get_user(
    pool: web::Data<PgPool>,
    path: web::Path<Uuid>,
) -> Result<HttpResponse> {
    let user_id = path.into_inner();

    let user = sqlx::query!(
        r#"
        SELECT u.id, u.username, u.email, u.full_name, u.bio, u.avatar_url, u.is_verified, u.created_at,
               COUNT(DISTINCT f1.follower_id) as "follower_count!",
               COUNT(DISTINCT f2.following_id) as "following_count!"
        FROM users u
        LEFT JOIN follows f1 ON u.id = f1.following_id
        LEFT JOIN follows f2 ON u.id = f2.follower_id
        WHERE u.id = $1
        GROUP BY u.id
        "#,
        user_id
    )
    .fetch_optional(pool.get_ref())
    .await;

    match user {
        Ok(Some(user)) => {
            let user_response = UserResponse {
                id: user.id,
                username: user.username,
                email: user.email,
                full_name: user.full_name,
                bio: user.bio,
                avatar_url: user.avatar_url,
                is_verified: user.is_verified.unwrap_or(false),
                follower_count: user.follower_count,
                following_count: user.following_count,
                created_at: user.created_at.unwrap(),
            };
            Ok(HttpResponse::Ok().json(ApiResponse::success(user_response)))
        }
        Ok(None) => Ok(HttpResponse::NotFound().json(ApiResponse::<()>::error(
            "User not found".to_string(),
        ))),
        Err(e) => {
            log::error!("Database error: {:?}", e);
            Ok(HttpResponse::InternalServerError().json(ApiResponse::<()>::error(
                "Database error".to_string(),
            )))
        }
    }
}

pub async fn get_profile(
    pool: web::Data<PgPool>,
    http_req: HttpRequest,
) -> Result<HttpResponse> {
    let user_id = match get_user_id_from_request(&http_req) {
        Some(id) => id,
        None => {
            return Ok(HttpResponse::Unauthorized().json(ApiResponse::<()>::error(
                "Authentication required".to_string(),
            )));
        }
    };

    get_user(pool, web::Path::from(user_id)).await
}

pub async fn update_profile(
    pool: web::Data<PgPool>,
    req: web::Json<UpdateUserRequest>,
    http_req: HttpRequest,
) -> Result<HttpResponse> {
    let user_id = match get_user_id_from_request(&http_req) {
        Some(id) => id,
        None => {
            return Ok(HttpResponse::Unauthorized().json(ApiResponse::<()>::error(
                "Authentication required".to_string(),
            )));
        }
    };

    if let Err(errors) = req.validate() {
        return Ok(HttpResponse::BadRequest().json(ApiResponse::<()>::error(
            format!("Validation error: {:?}", errors),
        )));
    }

    let updated_user = sqlx::query!(
        r#"
        UPDATE users SET
            full_name = COALESCE($2, full_name),
            bio = COALESCE($3, bio),
            avatar_url = COALESCE($4, avatar_url),
            updated_at = $5
        WHERE id = $1
        RETURNING id, username, email, full_name, bio, avatar_url, is_verified, created_at
        "#,
        user_id,
        req.full_name.as_deref(),
        req.bio.as_deref(),
        req.avatar_url.as_deref(),
        chrono::Utc::now()
    )
    .fetch_one(pool.get_ref())
    .await;

    match updated_user {
        Ok(user) => {
            // Get follower counts
            let counts = sqlx::query!(
                r#"
                SELECT COUNT(DISTINCT f1.follower_id) as "follower_count!",
                       COUNT(DISTINCT f2.following_id) as "following_count!"
                FROM users u
                LEFT JOIN follows f1 ON u.id = f1.following_id
                LEFT JOIN follows f2 ON u.id = f2.follower_id
                WHERE u.id = $1
                GROUP BY u.id
                "#,
                user_id
            )
            .fetch_one(pool.get_ref())
            .await;

            let user_response = match counts {
                Ok(counts) => UserResponse {
                    id: user.id,
                    username: user.username,
                    email: user.email,
                    full_name: user.full_name,
                    bio: user.bio,
                    avatar_url: user.avatar_url,
                    is_verified: user.is_verified.unwrap_or(false),
                    follower_count: counts.follower_count,
                    following_count: counts.following_count,
                    created_at: user.created_at.unwrap(),
                },
                Err(_) => UserResponse {
                    id: user.id,
                    username: user.username,
                    email: user.email,
                    full_name: user.full_name,
                    bio: user.bio,
                    avatar_url: user.avatar_url,
                    is_verified: user.is_verified.unwrap_or(false),
                    follower_count: 0,
                    following_count: 0,
                    created_at: user.created_at.unwrap(),
                },
            };

            Ok(HttpResponse::Ok().json(ApiResponse::success(user_response)))
        }
        Err(e) => {
            log::error!("Failed to update user: {:?}", e);
            Ok(HttpResponse::InternalServerError().json(ApiResponse::<()>::error(
                "Failed to update profile".to_string(),
            )))
        }
    }
}