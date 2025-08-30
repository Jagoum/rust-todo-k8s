use actix_web::{web, HttpRequest, HttpResponse, Result};
use chrono::Utc;
use sqlx::PgPool;
use uuid::Uuid;

use crate::middleware::auth::get_user_id_from_request;
use crate::models::ApiResponse;

pub async fn like_post(
    pool: web::Data<PgPool>,
    path: web::Path<Uuid>,
    http_req: HttpRequest,
) -> Result<HttpResponse> {
    let post_id = path.into_inner();
    let user_id = match get_user_id_from_request(&http_req) {
        Some(id) => id,
        None => {
            return Ok(HttpResponse::Unauthorized().json(ApiResponse::<()>::error(
                "Authentication required".to_string(),
            )));
        }
    };

    // Check if post exists
    let post_exists = sqlx::query!(
        "SELECT id FROM posts WHERE id = $1 AND is_published = true",
        post_id
    )
    .fetch_optional(pool.get_ref())
    .await;

    match post_exists {
        Ok(None) => {
            return Ok(HttpResponse::NotFound().json(ApiResponse::<()>::error(
                "Post not found".to_string(),
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

    // Check if user already liked the post
    let existing_like = sqlx::query!(
        "SELECT id FROM likes WHERE post_id = $1 AND user_id = $2",
        post_id,
        user_id
    )
    .fetch_optional(pool.get_ref())
    .await;

    match existing_like {
        Ok(Some(_)) => {
            return Ok(HttpResponse::Conflict().json(ApiResponse::<()>::error(
                "Post already liked".to_string(),
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

    // Create like
    let like_id = Uuid::new_v4();
    let result = sqlx::query!(
        r#"
        INSERT INTO likes (id, user_id, post_id, created_at)
        VALUES ($1, $2, $3, $4)
        "#,
        like_id,
        user_id,
        post_id,
        Utc::now()
    )
    .execute(pool.get_ref())
    .await;

    match result {
        Ok(_) => {
            // Get updated like count
            let like_count: (i64,) = sqlx::query_as(
                "SELECT COUNT(*) FROM likes WHERE post_id = $1"
            )
            .bind(post_id)
            .fetch_one(pool.get_ref())
            .await
            .unwrap_or((0,));

            #[derive(serde::Serialize)]
            struct LikeResponse {
                like_count: i64,
                is_liked: bool,
            }

            let response = LikeResponse {
                like_count: like_count.0,
                is_liked: true,
            };

            Ok(HttpResponse::Created().json(ApiResponse::success(response)))
        }
        Err(e) => {
            log::error!("Failed to create like: {:?}", e);
            Ok(HttpResponse::InternalServerError().json(ApiResponse::<()>::error(
                "Failed to like post".to_string(),
            )))
        }
    }
}

pub async fn unlike_post(
    pool: web::Data<PgPool>,
    path: web::Path<Uuid>,
    http_req: HttpRequest,
) -> Result<HttpResponse> {
    let post_id = path.into_inner();
    let user_id = match get_user_id_from_request(&http_req) {
        Some(id) => id,
        None => {
            return Ok(HttpResponse::Unauthorized().json(ApiResponse::<()>::error(
                "Authentication required".to_string(),
            )));
        }
    };

    let result = sqlx::query!(
        "DELETE FROM likes WHERE post_id = $1 AND user_id = $2",
        post_id,
        user_id
    )
    .execute(pool.get_ref())
    .await;

    match result {
        Ok(result) if result.rows_affected() > 0 => {
            // Get updated like count
            let like_count: (i64,) = sqlx::query_as(
                "SELECT COUNT(*) FROM likes WHERE post_id = $1"
            )
            .bind(post_id)
            .fetch_one(pool.get_ref())
            .await
            .unwrap_or((0,));

            #[derive(serde::Serialize)]
            struct LikeResponse {
                like_count: i64,
                is_liked: bool,
            }

            let response = LikeResponse {
                like_count: like_count.0,
                is_liked: false,
            };

            Ok(HttpResponse::Ok().json(ApiResponse::success(response)))
        }
        Ok(_) => Ok(HttpResponse::NotFound().json(ApiResponse::<()>::error(
            "Like not found".to_string(),
        ))),
        Err(e) => {
            log::error!("Failed to unlike post: {:?}", e);
            Ok(HttpResponse::InternalServerError().json(ApiResponse::<()>::error(
                "Failed to unlike post".to_string(),
            )))
        }
    }
}
