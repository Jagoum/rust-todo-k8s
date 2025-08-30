use actix_web::{web, HttpRequest, HttpResponse, Result};
use chrono::Utc;
use slug::slugify;
use sqlx::PgPool;
use uuid::Uuid;
use validator::Validate;

use crate::middleware::auth::{extract_optional_user_id, get_user_id_from_request};
use crate::models::{
    ApiResponse, CreatePostRequest, PaginatedResponse, PaginationParams, 
    Post, PostResponse, UpdatePostRequest, UserResponse
};

pub async fn create_post(
    pool: web::Data<PgPool>,
    req: web::Json<CreatePostRequest>,
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

    let post_id = Uuid::new_v4();
    let slug = slugify(&req.title);

    let post = sqlx::query_as!(
        Post,
        r#"
        INSERT INTO posts (id, title, slug, content, excerpt, cover_image, author_id, is_published, created_at, updated_at)
        VALUES ($1, $2, $3, $4, $5, $6, $7, false, $8, $8)
        RETURNING id, title, slug, content, excerpt, cover_image, author_id, is_published, published_at, created_at, updated_at
        "#,
        post_id,
        req.title,
        slug,
        req.content,
        req.excerpt,
        req.cover_image,
        user_id,
        Utc::now()
    )
    .fetch_one(pool.get_ref())
    .await;

    match post {
        Ok(post) => {
            // Handle tags if provided
            if let Some(tags) = &req.tags {
                for tag_name in tags {
                    let _ = add_tag_to_post(&pool, post.id, tag_name).await;
                }
            }

            let post_response = build_post_response(&pool, post, None).await?;
            Ok(HttpResponse::Created().json(ApiResponse::success(post_response)))
        }
        Err(e) => {
            log::error!("Failed to create post: {:?}", e);
            Ok(HttpResponse::InternalServerError().json(ApiResponse::<()>::error(
                "Failed to create post".to_string(),
            )))
        }
    }
}

pub async fn get_post(
    pool: web::Data<PgPool>,
    path: web::Path<Uuid>,
    http_req: HttpRequest,
) -> Result<HttpResponse> {
    let post_id = path.into_inner();
    let user_id = extract_optional_user_id(&http_req);

    let post = sqlx::query_as!(
        Post,
        "SELECT id, title, slug, content, excerpt, cover_image, author_id, is_published, published_at, created_at, updated_at FROM posts WHERE id = $1 AND is_published = true",
        post_id
    )
    .fetch_optional(pool.get_ref())
    .await;

    match post {
        Ok(Some(post)) => {
            let post_response = build_post_response(&pool, post, user_id).await?;
            Ok(HttpResponse::Ok().json(ApiResponse::success(post_response)))
        }
        Ok(None) => Ok(HttpResponse::NotFound().json(ApiResponse::<()>::error(
            "Post not found".to_string(),
        ))),
        Err(e) => {
            log::error!("Database error: {:?}", e);
            Ok(HttpResponse::InternalServerError().json(ApiResponse::<()>::error(
                "Database error".to_string(),
            )))
        }
    }
}

pub async fn get_posts(
    pool: web::Data<PgPool>,
    query: web::Query<PaginationParams>,
    http_req: HttpRequest,
) -> Result<HttpResponse> {
    let user_id = extract_optional_user_id(&http_req);
    let pagination = query.into_inner();
    let page = pagination.page.unwrap_or(1);
    let limit = pagination.limit.unwrap_or(20);
    let offset = (page - 1) * limit;

    // Get total count
    let total: (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM posts WHERE is_published = true"
    )
    .fetch_one(pool.get_ref())
    .await
    .map_err(|e| {
        log::error!("Database error: {:?}", e);
        actix_web::error::ErrorInternalServerError("Database error")
    })?;

    // Get posts
    let posts = sqlx::query_as!(
        Post,
        r#"
        SELECT id, title, slug, content, excerpt, cover_image, author_id, is_published, published_at, created_at, updated_at FROM posts
        WHERE is_published = true
        ORDER BY published_at DESC
        LIMIT $1 OFFSET $2
        "#,
        limit as i64,
        offset as i64
    )
    .fetch_all(pool.get_ref())
    .await;

    match posts {
        Ok(posts) => {
            let mut post_responses = Vec::new();
            for post in posts {
                let post_response = build_post_response(&pool, post, user_id).await?;
                post_responses.push(post_response);
            }

            let total_pages = (total.0 as f64 / limit as f64).ceil() as u32;

            let paginated_response = PaginatedResponse {
                data: post_responses,
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

pub async fn update_post(
    pool: web::Data<PgPool>,
    path: web::Path<Uuid>,
    req: web::Json<UpdatePostRequest>,
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

    // Check if post exists and user owns it
    let existing_post = sqlx::query!(
        "SELECT author_id FROM posts WHERE id = $1",
        post_id
    )
    .fetch_optional(pool.get_ref())
    .await;

    match existing_post {
        Ok(Some(post)) if post.author_id == user_id => {
            // For simplicity, let's use a more straightforward approach
            let updated_post = if req.title.is_some() || req.content.is_some() || req.excerpt.is_some() || req.cover_image.is_some() {
                sqlx::query_as!(
                    Post,
                    r#"
                    UPDATE posts SET
                        title = COALESCE($2, title),
                        slug = COALESCE($3, slug),
                        content = COALESCE($4, content),
                        excerpt = COALESCE($5, excerpt),
                        cover_image = COALESCE($6, cover_image),
                        updated_at = $7
                    WHERE id = $1
                    RETURNING id, title, slug, content, excerpt, cover_image, author_id, is_published, published_at, created_at, updated_at
                    "#,
                    post_id,
                    req.title.as_deref(),
                    req.title.as_ref().map(|t| slugify(t)),
                    req.content.as_deref(),
                    req.excerpt.as_deref(),
                    req.cover_image.as_deref(),
                    Utc::now()
                )
                .fetch_one(pool.get_ref())
                .await
            } else {
                return Ok(HttpResponse::BadRequest().json(ApiResponse::<()>::error(
                    "No fields to update".to_string(),
                )));
            };

            match updated_post {
                Ok(post) => {
                    // Handle tags if provided
                    if let Some(tags) = &req.tags {
                        // Remove existing tags
                        let _ = sqlx::query!(
                            "DELETE FROM post_tags WHERE post_id = $1",
                            post_id
                        )
                        .execute(pool.get_ref())
                        .await;

                        // Add new tags
                        for tag_name in tags {
                            let _ = add_tag_to_post(&pool, post.id, tag_name).await;
                        }
                    }

                    let post_response = build_post_response(&pool, post, Some(user_id)).await?;
                    Ok(HttpResponse::Ok().json(ApiResponse::success(post_response)))
                }
                Err(e) => {
                    log::error!("Failed to update post: {:?}", e);
                    Ok(HttpResponse::InternalServerError().json(ApiResponse::<()>::error(
                        "Failed to update post".to_string(),
                    )))
                }
            }
        }
        Ok(Some(_)) => Ok(HttpResponse::Forbidden().json(ApiResponse::<()>::error(
            "You don't have permission to update this post".to_string(),
        ))),
        Ok(None) => Ok(HttpResponse::NotFound().json(ApiResponse::<()>::error(
            "Post not found".to_string(),
        ))),
        Err(e) => {
            log::error!("Database error: {:?}", e);
            Ok(HttpResponse::InternalServerError().json(ApiResponse::<()>::error(
                "Database error".to_string(),
            )))
        }
    }
}

pub async fn delete_post(
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
        "DELETE FROM posts WHERE id = $1 AND author_id = $2",
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
            "Post not found or you don't have permission to delete it".to_string(),
        ))),
        Err(e) => {
            log::error!("Failed to delete post: {:?}", e);
            Ok(HttpResponse::InternalServerError().json(ApiResponse::<()>::error(
                "Failed to delete post".to_string(),
            )))
        }
    }
}

pub async fn publish_post(
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

    let post = sqlx::query_as!(
        Post,
        r#"
        UPDATE posts SET
            is_published = true,
            published_at = $3,
            updated_at = $3
        WHERE id = $1 AND author_id = $2
        RETURNING id, title, slug, content, excerpt, cover_image, author_id, is_published, published_at, created_at, updated_at
        "#,
        post_id,
        user_id,
        Utc::now()
    )
    .fetch_optional(pool.get_ref())
    .await;

    match post {
        Ok(Some(post)) => {
            let post_response = build_post_response(&pool, post, Some(user_id)).await?;
            Ok(HttpResponse::Ok().json(ApiResponse::success(post_response)))
        }
        Ok(None) => Ok(HttpResponse::NotFound().json(ApiResponse::<()>::error(
            "Post not found or you don't have permission to publish it".to_string(),
        ))),
        Err(e) => {
            log::error!("Failed to publish post: {:?}", e);
            Ok(HttpResponse::InternalServerError().json(ApiResponse::<()>::error(
                "Failed to publish post".to_string(),
            )))
        }
    }
}

pub async fn get_drafts(
    pool: web::Data<PgPool>,
    query: web::Query<PaginationParams>,
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

    let pagination = query.into_inner();
    let page = pagination.page.unwrap_or(1);
    let limit = pagination.limit.unwrap_or(20);
    let offset = (page - 1) * limit;

    let total: (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM posts WHERE author_id = $1 AND is_published = false"
    )
    .bind(user_id)
    .fetch_one(pool.get_ref())
    .await
    .map_err(|e| {
        log::error!("Database error: {:?}", e);
        actix_web::error::ErrorInternalServerError("Database error")
    })?;

    let posts = sqlx::query_as!(
        Post,
        r#"
        SELECT id, title, slug, content, excerpt, cover_image, author_id, is_published, published_at, created_at, updated_at FROM posts
        WHERE author_id = $1 AND is_published = false
        ORDER BY created_at DESC
        LIMIT $2 OFFSET $3
        "#,
        user_id,
        limit as i64,
        offset as i64
    )
    .fetch_all(pool.get_ref())
    .await;

    match posts {
        Ok(posts) => {
            let mut post_responses = Vec::new();
            for post in posts {
                let post_response = build_post_response(&pool, post, Some(user_id)).await?;
                post_responses.push(post_response);
            }

            let total_pages = (total.0 as f64 / limit as f64).ceil() as u32;

            let paginated_response = PaginatedResponse {
                data: post_responses,
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

pub async fn get_feed(
    pool: web::Data<PgPool>,
    query: web::Query<PaginationParams>,
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

    let pagination = query.into_inner();
    let page = pagination.page.unwrap_or(1);
    let limit = pagination.limit.unwrap_or(20);
    let offset = (page - 1) * limit;

    // Get posts from followed users
    let posts = sqlx::query_as!(
        Post,
        r#"
        SELECT p.id, p.title, p.slug, p.content, p.excerpt, p.cover_image, p.author_id, p.is_published, p.published_at, p.created_at, p.updated_at FROM posts p
        INNER JOIN follows f ON p.author_id = f.following_id
        WHERE f.follower_id = $1 AND p.is_published = true
        ORDER BY p.published_at DESC
        LIMIT $2 OFFSET $3
        "#,
        user_id,
        limit as i64,
        offset as i64
    )
    .fetch_all(pool.get_ref())
    .await;

    match posts {
        Ok(posts) => {
            let mut post_responses = Vec::new();
            for post in posts {
                let post_response = build_post_response(&pool, post, Some(user_id)).await?;
                post_responses.push(post_response);
            }

            let total: (i64,) = sqlx::query_as(
                r#"
                SELECT COUNT(*) FROM posts p
                INNER JOIN follows f ON p.author_id = f.following_id
                WHERE f.follower_id = $1 AND p.is_published = true
                "#
            )
            .bind(user_id)
            .fetch_one(pool.get_ref())
.await
            .unwrap_or((0,));

            let total_pages = (total.0 as f64 / limit as f64).ceil() as u32;

            let paginated_response = PaginatedResponse {
                data: post_responses,
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

// Helper functions
async fn build_post_response(
    pool: &PgPool,
    post: Post,
    current_user_id: Option<Uuid>,
) -> Result<PostResponse> {
    // Get author info
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
        post.author_id
    )
    .fetch_one(pool)
    .await
    .map_err(|e| actix_web::error::ErrorInternalServerError(e))?;

    // Get tags
    let tags = sqlx::query!(
        r#"
        SELECT t.name FROM tags t
        INNER JOIN post_tags pt ON t.id = pt.tag_id
        WHERE pt.post_id = $1
        "#,
        post.id
    )
    .fetch_all(pool)
    .await
    .map_err(|e| actix_web::error::ErrorInternalServerError(e))?;

    // Get like count
    let like_count: (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM likes WHERE post_id = $1"
    )
    .bind(post.id)
    .fetch_one(pool)
    .await
    .unwrap_or((0,));

    // Get comment count
    let comment_count: (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM comments WHERE post_id = $1"
    )
    .bind(post.id)
    .fetch_one(pool)
    .await
    .unwrap_or((0,));

    // Check if current user liked the post
    let is_liked = if let Some(user_id) = current_user_id {
        sqlx::query!(
            "SELECT id FROM likes WHERE post_id = $1 AND user_id = $2",
            post.id,
            user_id
        )
        .fetch_optional(pool)
        .await
        .unwrap_or(None)
        .is_some()
    } else {
        false
    };

    Ok(PostResponse {
        id: post.id,
        title: post.title,
        slug: post.slug,
        content: post.content,
        excerpt: post.excerpt,
        cover_image: post.cover_image,
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
        tags: tags.into_iter().map(|t| t.name).collect(),
        like_count: like_count.0,
        comment_count: comment_count.0,
        is_liked,
        is_published: post.is_published.unwrap_or(false),
        published_at: post.published_at,
        created_at: post.created_at.unwrap(),
        updated_at: post.updated_at.unwrap(),
    })
}

async fn add_tag_to_post(pool: &PgPool, post_id: Uuid, tag_name: &str) -> Result<(), sqlx::Error> {
    // Insert or get tag
    let tag = sqlx::query!(
        r#"
        INSERT INTO tags (id, name, created_at)
        VALUES ($1, $2, $3)
        ON CONFLICT (name) DO UPDATE SET name = EXCLUDED.name
        RETURNING id
        "#,
        Uuid::new_v4(),
        tag_name,
        Utc::now()
    )
    .fetch_one(pool)
    .await?;

    // Link tag to post
    sqlx::query!(
        "INSERT INTO post_tags (post_id, tag_id) VALUES ($1, $2) ON CONFLICT DO NOTHING",
        post_id,
        tag.id
    )
    .execute(pool)
    .await?;

    Ok(())
}