use actix_web::{web, HttpRequest, HttpResponse, Result};
use chrono::Utc;
use sqlx::PgPool;
use uuid::Uuid;

use crate::middleware::auth::get_user_id_from_request;
use crate::models::{ApiResponse, PaginatedResponse, PaginationParams, UserResponse};

pub async fn follow_user(
    pool: web::Data<PgPool>,
    path: web::Path<Uuid>,
    http_req: HttpRequest,
) -> Result<HttpResponse> {
    let following_id = path.into_inner();
    let follower_id = match get_user_id_from_request(&http_req) {
        Some(id) => id,
        None => {
            return Ok(HttpResponse::Unauthorized().json(ApiResponse::<()>::error(
                "Authentication required".to_string(),
            )));
        }
    };

    // Can't follow yourself
    if follower_id == following_id {
        return Ok(HttpResponse::BadRequest().json(ApiResponse::<()>::error(
            "You cannot follow yourself".to_string(),
        )));
    }

    // Check if user exists
    let user_exists = sqlx::query!(
        "SELECT id FROM users WHERE id = $1",
        following_id
    )
    .fetch_optional(pool.get_ref())
    .await;

    match user_exists {
        Ok(None) => {
            return Ok(HttpResponse::NotFound().json(ApiResponse::<()>::error(
                "User not found".to_string(),
            )));
        }
        Err(e) => {
            log::error!("Database error: {:?}", e);
            return Ok(HttpResponse::InternalServerError().json(ApiResponse::<()>::error(
                "Database error".to_string(),
            )));
        }
        Ok(Some(_)) => {}
    }

    // Check if already following
    let existing_follow = sqlx::query!(
        "SELECT id FROM follows WHERE follower_id = $1 AND following_id = $2",
        follower_id,
        following_id
    )
    .fetch_optional(pool.get_ref())
    .await;

    match existing_follow {
        Ok(Some(_)) => {
            return Ok(HttpResponse::Conflict().json(ApiResponse::<()>::error(
                "Already following this user".to_string(),
            )));
        }
        Err(e) => {
            log::error!("Database error: {:?}", e);
            return Ok(HttpResponse::InternalServerError().json(ApiResponse::<()>::error(
                "Database error".to_string(),
            )));
        }
        Ok(None) => {}
    }

    // Create follow relationship
    let follow_id = Uuid::new_v4();
    let result = sqlx::query!(
        r#"
        INSERT INTO follows (id, follower_id, following_id, created_at)
        VALUES ($1, $2, $3, $4)
        "#,
        follow_id,
        follower_id,
        following_id,
        Utc::now()
    )
    .execute(pool.get_ref())
    .await;

    match result {
        Ok(_) => {
            #[derive(serde::Serialize)]
            struct FollowResponse {
                following: bool,
            }

            Ok(HttpResponse::Created().json(ApiResponse::success(FollowResponse {
                following: true,
            })))
        }
        Err(e) => {
            log::error!("Failed to create follow: {:?}", e);
            Ok(HttpResponse::InternalServerError().json(ApiResponse::<()>::error(
                "Failed to follow user".to_string(),
            )))
        }
    }
}

pub async fn unfollow_user(
    pool: web::Data<PgPool>,
    path: web::Path<Uuid>,
    http_req: HttpRequest,
) -> Result<HttpResponse> {
    let following_id = path.into_inner();
    let follower_id = match get_user_id_from_request(&http_req) {
        Some(id) => id,
        None => {
            return Ok(HttpResponse::Unauthorized().json(ApiResponse::<()>::error(
                "Authentication required".to_string(),
            )));
        }
    };

    let result = sqlx::query!(
        "DELETE FROM follows WHERE follower_id = $1 AND following_id = $2",
        follower_id,
        following_id
    )
    .execute(pool.get_ref())
    .await;

    match result {
        Ok(result) if result.rows_affected() > 0 => {
            #[derive(serde::Serialize)]
            struct FollowResponse {
                following: bool,
            }

            Ok(HttpResponse::Ok().json(ApiResponse::success(FollowResponse {
                following: false,
            })))
        }
        Ok(_) => Ok(HttpResponse::NotFound().json(ApiResponse::<()>::error(
            "Follow relationship not found".to_string(),
        ))),
        Err(e) => {
            log::error!("Failed to unfollow user: {:?}", e);
            Ok(HttpResponse::InternalServerError().json(ApiResponse::<()>::error(
                "Failed to unfollow user".to_string(),
            )))
        }
    }
}

pub async fn get_followers(
    pool: web::Data<PgPool>,
    path: web::Path<Uuid>,
    query: web::Query<PaginationParams>,
) -> Result<HttpResponse> {
    let user_id = path.into_inner();
    let pagination = query.into_inner();
    let page = pagination.page.unwrap_or(1);
    let limit = pagination.limit.unwrap_or(20);
    let offset = (page - 1) * limit;

    // Get total count
    let total: (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM follows WHERE following_id = $1"
    )
    .bind(user_id)
    .fetch_one(pool.get_ref())
    .await
    .map_err(|e| {
        log::error!("Database error: {:?}", e);
        actix_web::error::ErrorInternalServerError("Database error")
    })?;

    // Get followers
    let followers = sqlx::query!(
        r#"
        SELECT u.id, u.username, u.email, u.full_name, u.bio, u.avatar_url, u.is_verified, u.created_at,
               COUNT(DISTINCT f1.follower_id) as "follower_count!",
               COUNT(DISTINCT f2.following_id) as "following_count!"
        FROM users u
        INNER JOIN follows f ON u.id = f.follower_id
        LEFT JOIN follows f1 ON u.id = f1.following_id
        LEFT JOIN follows f2 ON u.id = f2.follower_id
        WHERE f.following_id = $1
        GROUP BY u.id, f.created_at
        ORDER BY f.created_at DESC
        LIMIT $2 OFFSET $3
        "#,
        user_id,
        limit as i64,
        offset as i64
    )
    .fetch_all(pool.get_ref())
    .await;

    match followers {
        Ok(followers) => {
            let user_responses: Vec<UserResponse> = followers
                .into_iter()
                .map(|user| UserResponse {
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
                })
                .collect();

            let total_pages = (total.0 as f64 / limit as f64).ceil() as u32;

            let paginated_response = PaginatedResponse {
                data: user_responses,
                total: total.0,
                page,
                limit,
                total_pages,
            };

            Ok(HttpResponse::Ok().json(ApiResponse::success(paginated_response)))
        }
        Err(e) => {
            log::error!("Database error: {:?}", e);
            Ok(HttpResponse::InternalServerError().json(ApiResponse::<()>::error(
                "Database error".to_string(),
            )))
        }
    }
}

pub async fn get_following(
    pool: web::Data<PgPool>,
    path: web::Path<Uuid>,
    query: web::Query<PaginationParams>,
) -> Result<HttpResponse> {
    let user_id = path.into_inner();
    let pagination = query.into_inner();
    let page = pagination.page.unwrap_or(1);
    let limit = pagination.limit.unwrap_or(20);
    let offset = (page - 1) * limit;

    // Get total count
    let total: (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM follows WHERE follower_id = $1"
    )
    .bind(user_id)
    .fetch_one(pool.get_ref())
    .await
    .map_err(|e| {
        log::error!("Database error: {:?}", e);
        actix_web::error::ErrorInternalServerError("Database error")
    })?;

    // Get following
    let following = sqlx::query!(
        r#"
        SELECT u.id, u.username, u.email, u.full_name, u.bio, u.avatar_url, u.is_verified, u.created_at,
               COUNT(DISTINCT f1.follower_id) as "follower_count!",
               COUNT(DISTINCT f2.following_id) as "following_count!"
        FROM users u
        INNER JOIN follows f ON u.id = f.following_id
        LEFT JOIN follows f1 ON u.id = f1.following_id
        LEFT JOIN follows f2 ON u.id = f2.follower_id
        WHERE f.follower_id = $1
        GROUP BY u.id, f.created_at
        ORDER BY f.created_at DESC
        LIMIT $2 OFFSET $3
        "#,
        user_id,
        limit as i64,
        offset as i64
    )
    .fetch_all(pool.get_ref())
    .await;

    match following {
        Ok(following) => {
            let user_responses: Vec<UserResponse> = following
                .into_iter()
                .map(|user| UserResponse {
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
                })
                .collect();

            let total_pages = (total.0 as f64 / limit as f64).ceil() as u32;

            let paginated_response = PaginatedResponse {
                data: user_responses,
                total: total.0,
                page,
                limit,
                total_pages,
            };

            Ok(HttpResponse::Ok().json(ApiResponse::success(paginated_response)))
        }
        Err(e) => {
            log::error!("Database error: {:?}", e);
            Ok(HttpResponse::InternalServerError().json(ApiResponse::<()>::error(
                "Database error".to_string(),
            )))
        }
    }
}
