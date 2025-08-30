use actix_web::{web, HttpRequest, HttpResponse, Result};
use sqlx::PgPool;
use uuid::Uuid;

use crate::models::{ApiResponse, PaginatedResponse, PaginationParams, Post, PostResponse, Tag, UserResponse};
use crate::middleware::auth::extract_optional_user_id;

pub async fn get_tags(
    pool: web::Data<PgPool>,
    query: web::Query<PaginationParams>,
) -> Result<HttpResponse> {
    let pagination = query.into_inner();
    let page = pagination.page.unwrap_or(1);
    let limit = pagination.limit.unwrap_or(20);
    let offset = (page - 1) * limit;

    let tags = sqlx::query_as!(
        Tag,
        "SELECT id, name, created_at FROM tags ORDER BY name ASC LIMIT $1 OFFSET $2",
        limit as i64,
        offset as i64
    )
    .fetch_all(pool.get_ref())
    .await;

    match tags {
        Ok(tags) => {
            let total: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM tags")
                .fetch_one(pool.get_ref())
                .await
                .unwrap_or((0,));

            let total_pages = (total.0 as f64 / limit as f64).ceil() as u32;

            let paginated_response = PaginatedResponse {
                data: tags,
                total: total.0,
                page,
                limit,
                total_pages,
            };
            Ok(HttpResponse::Ok().json(ApiResponse::success(paginated_response)))
        }
        Err(e) => {
            log::error!("Failed to get tags: {:?}", e);
            Ok(HttpResponse::InternalServerError()
                .json(ApiResponse::<()>::error("Failed to get tags".to_string())))
        }
    }
}

pub async fn get_posts_by_tag(
    pool: web::Data<PgPool>,
    path: web::Path<String>,
    query: web::Query<PaginationParams>,
    http_req: HttpRequest,
) -> Result<HttpResponse> {
    let tag_name = path.into_inner();
    let user_id = extract_optional_user_id(&http_req);
    let pagination = query.into_inner();
    let page = pagination.page.unwrap_or(1);
    let limit = pagination.limit.unwrap_or(20);
    let offset = (page - 1) * limit;

    let posts = sqlx::query_as!(
        Post,
        r#"
        SELECT p.id, p.title, p.slug, p.content, p.excerpt, p.cover_image, p.author_id, p.is_published, p.published_at, p.created_at, p.updated_at FROM posts p
        INNER JOIN post_tags pt ON p.id = pt.post_id
        INNER JOIN tags t ON pt.tag_id = t.id
        WHERE t.name = $1 AND p.is_published = true
        ORDER BY p.published_at DESC
        LIMIT $2 OFFSET $3
        "#,
        tag_name,
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

            let total: (i64,) = sqlx::query_as(
                r#"
                SELECT COUNT(*) FROM posts p
                INNER JOIN post_tags pt ON p.id = pt.post_id
                INNER JOIN tags t ON pt.tag_id = t.id
                WHERE t.name = $1 AND p.is_published = true
                "#,
            )
            .bind(&tag_name)
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
            log::error!("Failed to get posts by tag: {:?}", e);
            Ok(HttpResponse::InternalServerError()
                .json(ApiResponse::<()>::error("Failed to get posts by tag".to_string())))
        }
    }
}

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