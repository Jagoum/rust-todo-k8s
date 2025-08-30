use actix_cors::Cors;
use actix_web::{web, App, HttpServer, middleware::Logger};
use sqlx::postgres::PgPoolOptions;
use std::env;

mod models;
mod handlers;
mod middleware;
mod utils;

use handlers::{posts, users, comments, likes, follows, tags};
use middleware::{auth};

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init();
    
    let database_url = env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgresql://postgres:password@localhost:5432/blog_db".to_string());
    
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await
        .expect("Failed to connect to database");
    
    // Run migrations
    sqlx::migrate!("./migrations")
        .run(&pool)
        .await
        .expect("Failed to run migrations");
    
    println!("Starting blog backend server on http://localhost:8080");
    
    HttpServer::new(move || {
        let cors = Cors::default()
            .allowed_origin("http://localhost:3000")
            .allowed_methods(vec!["GET", "POST", "PUT", "DELETE", "PATCH"])
            .allowed_headers(vec!["Authorization", "Content-Type"])
            .supports_credentials();
            
        App::new()
            .app_data(web::Data::new(pool.clone()))
            .wrap(Logger::default())
            .wrap(cors)
            .service(
                web::scope("/api/v1")
                    // Auth routes
                    .service(
                        web::scope("/auth")
                            .route("/register", web::post().to(auth::register))
                            .route("/login", web::post().to(auth::login))
                            .route("/refresh", web::post().to(auth::refresh_token))
                    )
                    // User routes
                    .service(
                        web::scope("/users")
                            .route("/{user_id}", web::get().to(users::get_user))
                            .route("/profile", web::get().to(users::get_profile))
                            .route("/profile", web::put().to(users::update_profile))
                            .route("/{user_id}/follow", web::post().to(follows::follow_user))
                            .route("/{user_id}/unfollow", web::delete().to(follows::unfollow_user))
                            .route("/{user_id}/followers", web::get().to(follows::get_followers))
                            .route("/{user_id}/following", web::get().to(follows::get_following))
                    )
                    // Post routes
                    .service(
                        web::scope("/posts")
                            .route("", web::get().to(posts::get_posts))
                            .route("", web::post().to(posts::create_post))
                            .route("/{post_id}", web::get().to(posts::get_post))
                            .route("/{post_id}", web::put().to(posts::update_post))
                            .route("/{post_id}", web::delete().to(posts::delete_post))
                            .route("/{post_id}/publish", web::patch().to(posts::publish_post))
                            .route("/{post_id}/like", web::post().to(likes::like_post))
                            .route("/{post_id}/unlike", web::delete().to(likes::unlike_post))
                            .route("/drafts", web::get().to(posts::get_drafts))
                            .route("/feed", web::get().to(posts::get_feed))
                    )
                    // Comment routes
                    .service(
                        web::scope("/posts/{post_id}/comments")
                            .route("", web::get().to(comments::get_comments))
                            .route("", web::post().to(comments::create_comment))
                            .route("/{comment_id}", web::put().to(comments::update_comment))
                            .route("/{comment_id}", web::delete().to(comments::delete_comment))
                    )
                    // Tag routes
                    .service(
                        web::scope("/tags")
                            .route("", web::get().to(tags::get_tags))
                            .route("/{tag_name}/posts", web::get().to(tags::get_posts_by_tag))
                    )
            )
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}