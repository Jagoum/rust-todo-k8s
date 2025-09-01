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

mod config;
mod db;
mod models;
mod auth;
mod routes;

use models::AppState;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenv().ok();
    tracing_subscriber::fmt()
        .with_max_level(Level::INFO)
        .with_target(false)
        .compact()
        .init();

    let cfg = config::Config::from_env()?;
    let pool = db::connect_pool(&cfg.database_url).await?;
    db::init_db(&pool).await?;

    let state = AppState { pool, jwt_secret: cfg.jwt_secret.clone() };

    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    let public = Router::new()
        .route("/healthz", get(|| async { "ok" }))
        .route("/register", post(routes::public::register))
        .route("/login", post(routes::public::login));

    let protected = Router::new()
        .route("/todos", get(routes::todos::list_todos).post(routes::todos::create_todo))
        .route("/todos/:id", put(routes::todos::update_todo).delete(routes::todos::delete_todo))
        .layer(from_fn_with_state(state.clone(), auth::auth_middleware));

    let app = Router::new()
        .merge(public)
        .merge(protected)
        .with_state(state)
        .layer(cors)
        .layer(TraceLayer::new_for_http());

    let addr: SocketAddr = format!("{}:{}", cfg.host, cfg.port).parse().unwrap();
    info!("listening on http://{}", addr);
    let listener = TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;
    Ok(())
}