//! # Todo Application Backend
//!
//! A RESTful API server built with Axum for managing todo items with user authentication.
//! 
//! ## Features
//! - User registration and JWT-based authentication
//! - CRUD operations for todo items
//! - PostgreSQL database integration with SQLx
//! - CORS support for frontend integration
//! - Request tracing and logging
//!
//! ## Architecture
//! - `config`: Application configuration management
//! - `db`: Database connection and initialization
//! - `models`: Data structures and application state
//! - `auth`: JWT authentication and middleware
//! - `routes`: HTTP route handlers (public and protected)

use std::net::SocketAddr;

use axum::{
    middleware::from_fn_with_state,
    routing::{get, post, put},
    Router,
};
use dotenvy::dotenv;
use tokio::net::TcpListener;
use tower_http::{cors::{Any, CorsLayer}, trace::TraceLayer};
use tracing::{info, Level};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

mod config;
mod db;
mod models;
mod auth;
mod routes;

use models::AppState;

/// Main application entry point
/// 
/// Initializes the web server with:
/// - Environment configuration loading
/// - Database connection and schema setup
/// - JWT authentication middleware
/// - CORS and request tracing
/// - Public and protected route registration
/// 
/// # Returns
/// 
/// Returns `Ok(())` on successful server shutdown, or an error if startup fails.
/// 
/// # Errors
/// 
/// This function will return an error if:
/// - Environment variables are missing or invalid
/// - Database connection fails
/// - Server binding fails
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Load environment variables
    dotenv().ok();
    
    // Initialize structured logging with JSON format for production
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "info,sqlx=warn,tower_http=debug".into())
        )
        .with(tracing_subscriber::fmt::layer().json())
        .init();

    info!("🚀 Starting Todo Application Backend");

    // Load configuration
    let cfg = config::Config::from_env()?;
    info!("📋 Configuration loaded successfully");
    info!("🌐 Server will bind to {}:{}", cfg.host, cfg.port);

    // Connect to database
    info!("🔌 Connecting to database...");
    let pool = db::connect_pool(&cfg.database_url).await?;
    info!("✅ Database connection established");

    // Initialize database schema
    info!("🗄️ Initializing database schema...");
    db::init_db(&pool).await?;
    info!("✅ Database schema initialized");

    let state = AppState { pool, jwt_secret: cfg.jwt_secret.clone() };

    // Configure CORS
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    // Setup public routes (no authentication required)
    let public = Router::new()
        .route("/healthz", get(|| async { 
            info!("Health check requested");
            "ok" 
        }))
        .route("/register", post(routes::public::register))
        .route("/login", post(routes::public::login));

    // Setup protected routes (authentication required)
    let protected = Router::new()
        .route("/todos", get(routes::todos::list_todos).post(routes::todos::create_todo))
        .route("/todos/:id", put(routes::todos::update_todo).delete(routes::todos::delete_todo))
        .layer(from_fn_with_state(state.clone(), auth::auth_middleware));

    // Combine all routes with middleware
    let app = Router::new()
        .merge(public)
        .merge(protected)
        .with_state(state)
        .layer(cors)
        .layer(
            TraceLayer::new_for_http()
                .make_span_with(tower_http::trace::DefaultMakeSpan::new().level(Level::INFO))
                .on_response(tower_http::trace::DefaultOnResponse::new().level(Level::INFO))
        );

    // Start server
    let addr: SocketAddr = format!("{}:{}", cfg.host, cfg.port).parse().unwrap();
    info!("🎯 Server listening on http://{}", addr);
    
    let listener = TcpListener::bind(addr).await?;
    info!("🚀 Todo Application Backend started successfully");
    
    axum::serve(listener, app).await?;
    Ok(())
}