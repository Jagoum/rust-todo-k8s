//! # Database Connection and Schema Management
//!
//! Handles PostgreSQL database connections using SQLx and manages database schema initialization.
//! Creates connection pools for efficient database access and sets up required tables.

use anyhow::Context;
use sqlx::{postgres::PgPoolOptions, PgPool};
use tracing::{info, instrument, warn, error};
use tokio::time::{sleep, Duration};

/// Create a PostgreSQL connection pool
/// 
/// Establishes a connection pool to the PostgreSQL database with optimized settings
/// for the application's expected load.
/// 
/// # Arguments
/// 
/// * `db_url` - PostgreSQL connection string (e.g., "postgres://user:pass@host:port/db")
/// 
/// # Returns
/// 
/// Returns a `PgPool` instance configured with:
/// - Maximum 10 concurrent connections
/// - Automatic connection management
/// - Connection validation and retry logic
/// 
/// # Errors
/// 
/// Returns an error if:
/// - Database URL is malformed
/// - Database server is unreachable
/// - Authentication fails
/// - Database does not exist
/// 
/// # Examples
/// 
/// ```rust
/// let pool = connect_pool("postgres://user:pass@localhost:5432/mydb").await?;
/// ```
#[instrument(skip(db_url), fields(db_url = %db_url.split('@').last().unwrap_or("***")))]
pub async fn connect_pool(db_url: &str) -> anyhow::Result<PgPool> {
    info!("Creating database connection pool with max 10 connections");
    
    let max_retries = 10; // 30 retries = ~60 seconds with 2 second intervals
    let mut retry_count = 0;
    
    loop {
        match PgPoolOptions::new()
            .max_connections(10)
            .connect(db_url)
            .await
        {
            Ok(pool) => {
                info!("Database connection pool created successfully");
                return Ok(pool);
            }
            Err(e) => {
                retry_count += 1;
                if retry_count >= max_retries {
                    error!("Failed to connect to database after {} retries", max_retries);
                    return Err(e).context("connect to Postgres after retries");
                }
                
                warn!("Database connection failed (attempt {}/{}): {}. Retrying in 2 seconds...", 
                    retry_count, max_retries, e);
                sleep(Duration::from_secs(2)).await;
            }
        }
    }
}

/// Initialize database schema
/// 
/// Creates all required tables for the application if they don't already exist.
/// This function is idempotent and safe to run multiple times.
/// 
/// # Database Schema
/// 
/// Creates two main tables:
/// 
/// ## Users Table
/// - `id`: UUID primary key
/// - `username`: Unique text identifier
/// - `password_hash`: Argon2 hashed password
/// - `created_at`: Timestamp with timezone
/// 
/// ## Todos Table
/// - `id`: UUID primary key
/// - `user_id`: Foreign key to users table (CASCADE DELETE)
/// - `title`: Todo item description
/// - `completed`: Boolean completion status
/// - `created_at`: Timestamp with timezone
/// 
/// # Arguments
/// 
/// * `pool` - Database connection pool
/// 
/// # Returns
/// 
/// Returns `Ok(())` if schema initialization succeeds.
/// 
/// # Errors
/// 
/// Returns an error if:
/// - Database connection fails
/// - SQL execution fails
/// - Insufficient database permissions
/// 
/// # Examples
/// 
/// ```rust
/// let pool = connect_pool(&database_url).await?;
/// init_db(&pool).await?;
/// ```
#[instrument(skip(pool))]
pub async fn init_db(pool: &PgPool) -> anyhow::Result<()> {
    info!("Initializing database schema");
    
    // Create users table with UUID primary key and unique username constraint
    info!("Creating users table if not exists");
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS users (
            id uuid PRIMARY KEY,
            username TEXT NOT NULL UNIQUE,
            password_hash TEXT NOT NULL,
            created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
        );
        "#,
    )
    .execute(pool)
    .await
    .context("Failed to create users table")?;

    // Create todos table with foreign key relationship to users
    info!("Creating todos table if not exists");
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS todos (
            id uuid PRIMARY KEY,
            user_id uuid NOT NULL REFERENCES users(id) ON DELETE CASCADE,
            title TEXT NOT NULL,
            completed BOOLEAN NOT NULL DEFAULT false,
            created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
        );
        "#,
    )
    .execute(pool)
    .await
    .context("Failed to create todos table")?;

    info!("Database schema initialization completed successfully");
    Ok(())
}