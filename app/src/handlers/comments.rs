use actix_web::{web, HttpRequest, HttpResponse, Result};
use chrono::Utc;
use sqlx::PgPool;
use uuid::Uuid;
use validator::Validate;

use crate::middleware::auth::get_user_id_from_request;
use crate::models::{ApiResponse, Comment, CommentResponse, CreateCommentRequest, UserResponse};

pub async fn get_comments(
    pool: web::Data<PgPool>,
    path: web::Path<Uuid>,
) -> Result<HttpResponse> {
    let post_id = path.into_inner();

    let comments = sqlx::query_as!(
        Comment,
        "SELECT id, content, post_id, author_id, parent_id, created_at, updated_at FROM comments WHERE post_id = $1 ORDER BY created_at ASC",
        post_id
    )
    .fetch_all(pool.get_ref())
    .await;

    match comments {
        Ok(comments) => {
            let mut comment_responses = Vec::new();

            // Build tree structure (simplified - only handles one level of nesting)
            let mut root_comments = Vec::new();
            let mut reply_map: std::collections::HashMap<Uuid, Vec<CommentResponse>> = std::collections::HashMap::new();

            for comment in comments {
                let comment_response = build_comment_response(&pool, comment).await?;

                if comment_response.parent_id.is_some() {
                    let parent_id = comment_response.parent_id.unwrap();
                    reply_map.entry(parent_id).or_insert_with(Vec::new).push(comment_response);
                } else {
                    root_comments.push(comment_response);
                }
            }

            // Attach replies to root comments
            for mut comment in root_comments {
                if let Some(replies) = reply_map.get(&comment.id) {
                    comment.replies = replies.clone();
                } else {
                    comment.replies = Vec::new();
                }
                comment_responses.push(comment);
            }

            Ok(HttpResponse::Ok().json(ApiResponse::success(comment_responses)))
        }
        Err(e) => {
            log::error!("Database error: {:?}", e);
            Ok(HttpResponse::InternalServerError().json(ApiResponse::<()>::error(
                "Database error".to_string(),
            )))
        }
    }
}

pub async fn create_comment(
    pool: web::Data<PgPool>,
    path: web::Path<Uuid>,
    req: web::Json<CreateCommentRequest>,
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

    if let Err(errors) = req.validate() {
        return Ok(HttpResponse::BadRequest().json(ApiResponse::<()>::error(
            format!("Validation error: {:?}", errors),
        )));
    }

    // Check if post exists
    let post_exists = sqlx::query!(
        "SELECT id FROM posts WHERE id = $1",
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

    // Check if parent comment exists (if provided)
    if let Some(parent_id) = req.parent_id {
        let parent_exists = sqlx::query!(
            "SELECT id FROM comments WHERE id = $1 AND post_id = $2",
            parent_id,
            post_id
        )
        .fetch_optional(pool.get_ref())
        .await;

        match parent_exists {
            Ok(None) => {
                return Ok(HttpResponse::BadRequest().json(ApiResponse::<()>::error(
                    "Parent comment not found".to_string(),
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
    }

    let comment_id = Uuid::new_v4();
    let comment = sqlx::query_as!(
        Comment,
        r#"
        INSERT INTO comments (id, content, post_id, author_id, parent_id, created_at, updated_at)
        VALUES ($1, $2, $3, $4, $5, $6, $6)
        RETURNING id, content, post_id, author_id, parent_id, created_at, updated_at
        "#,
        comment_id,
        req.content,
        post_id,
        user_id,
        req.parent_id,
        Utc::now()
    )
    .fetch_one(pool.get_ref())
    .await;

    match comment {
        Ok(comment) => {
            let comment_response = build_comment_response(&pool, comment).await?;
            Ok(HttpResponse::Created().json(ApiResponse::success(comment_response)))
        }
        Err(e) => {
            log::error!("Failed to create comment: {:?}", e);
            Ok(HttpResponse::InternalServerError().json(ApiResponse::<()>::error(
                "Failed to create comment".to_string(),
            )))
        }
    }
}

pub async fn update_comment(
    pool: web::Data<PgPool>,
    path: web::Path<(Uuid, Uuid)>,
    req: web::Json<CreateCommentRequest>,
    http_req: HttpRequest,
) -> Result<HttpResponse> {
    let (post_id, comment_id) = path.into_inner();
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

    let comment = sqlx::query_as!(
        Comment,
        r#"
        UPDATE comments SET
            content = $4,
            updated_at = $5
        WHERE id = $1 AND post_id = $2 AND author_id = $3
        RETURNING id, content, post_id, author_id, parent_id, created_at, updated_at
        "#,
        comment_id,
        post_id,
        user_id,
        req.content,
        Utc::now()
    )
    .fetch_optional(pool.get_ref())
    .await;

    match comment {
        Ok(Some(comment)) => {
            let comment_response = build_comment_response(&pool, comment).await?;
            Ok(HttpResponse::Ok().json(ApiResponse::success(comment_response)))
        }
        Ok(None) => Ok(HttpResponse::NotFound().json(ApiResponse::<()>::error(
            "Comment not found or you don't have permission to update it".to_string(),
        ))),
        Err(e) => {
            log::error!("Failed to update comment: {:?}", e);
            Ok(HttpResponse::InternalServerError().json(ApiResponse::<()>::error(
                "Failed to update comment".to_string(),
            )))
        }
    }
}

pub async fn delete_comment(
    pool: web::Data<PgPool>,
    path: web::Path<(Uuid, Uuid)>,
    http_req: HttpRequest,
) -> Result<HttpResponse> {
    let (post_id, comment_id) = path.into_inner();
    let user_id = match get_user_id_from_request(&http_req) {
        Some(id) => id,
        None => {
            return Ok(HttpResponse::Unauthorized().json(ApiResponse::<()>::error(
                "Authentication required".to_string(),
            )));
        }
    };

    let result = sqlx::query!(
        "DELETE FROM comments WHERE id = $1 AND post_id = $2 AND author_id = $3",
        comment_id,
        post_id,
        user_id
    )
    .execute(pool.get_ref())
    .await;

    match result {
        Ok(result) if result.rows_affected() > 0 => {
            Ok(HttpResponse::NoContent().finish())
        }
        Ok(_) => Ok(HttpResponse::NotFound().json(ApiResponse::<()>::error(
            "Comment not found or you don't have permission to delete it".to_string(),
        ))),
        Err(e) => {
            log::error!("Failed to delete comment: {:?}", e);
            Ok(HttpResponse::InternalServerError().json(ApiResponse::<()>::error(
                "Failed to delete comment".to_string(),
            )))
        }
    }
}

async fn build_comment_response(
    pool: &PgPool,
    comment: Comment,
) -> Result<CommentResponse> {
    let author = sqlx::query!(
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
        comment.author_id
    )
    .fetch_one(pool)
    .await
    .map_err(|e| actix_web::error::ErrorInternalServerError(e))?;

    Ok(CommentResponse {
        id: comment.id,
        content: comment.content,
        author: UserResponse {
            id: author.id,
            username: author.username,
            email: author.email,
            full_name: author.full_name,
            bio: author.bio,
            avatar_url: author.avatar_url,
            is_verified: author.is_verified.unwrap_or(false),
            follower_count: author.follower_count,
            following_count: author.following_count,
            created_at: author.created_at.unwrap(),
        },
        parent_id: comment.parent_id,
        replies: Vec::new(), // Will be populated by the calling function
        created_at: comment.created_at.unwrap(),
        updated_at: comment.updated_at.unwrap(),
    })
}