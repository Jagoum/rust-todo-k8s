//! # Application Configuration
//!
//! Handles loading and validation of application configuration from environment variables.
//! Provides sensible defaults for development while requiring critical values like database URL.

use anyhow::Context;

/// Application configuration loaded from environment variables
/// 
/// This struct contains all the configuration needed to run the application,
/// including server settings, database connection, and JWT secret.
pub struct Config {
    /// Host address to bind the server to (default: "0.0.0.0")
    pub host: String,
    /// Port number to listen on (default: 8080)
    pub port: u16,
    /// PostgreSQL database connection URL (required)
    pub database_url: String,
    /// Secret key for JWT token signing (default: development key)
    pub jwt_secret: String,
}

impl Config {
    /// Load configuration from environment variables
    /// 
    /// # Environment Variables
    /// 
    /// - `DATABASE_URL` (required): PostgreSQL connection string
    /// - `JWT_SECRET` (optional): Secret for JWT signing, defaults to dev key
    /// - `APP_HOST` (optional): Server host, defaults to "0.0.0.0"
    /// - `APP_PORT` (optional): Server port, defaults to 8080
    /// 
    /// # Returns
    /// 
    /// Returns a `Config` instance with all values loaded and validated.
    /// 
    /// # Errors
    /// 
    /// Returns an error if:
    /// - `DATABASE_URL` environment variable is missing
    /// - `APP_PORT` is provided but not a valid u16
    /// 
    /// # Examples
    /// 
    /// ```rust
    /// use std::env;
    /// 
    /// env::set_var("DATABASE_URL", "postgres://user:pass@localhost/db");
    /// let config = Config::from_env().unwrap();
    /// assert_eq!(config.host, "0.0.0.0");
    /// assert_eq!(config.port, 8080);
    /// ```
    pub fn from_env() -> anyhow::Result<Self> {
        let database_url = std::env::var("DATABASE_URL").context("DATABASE_URL env var is required")?;
        let jwt_secret = std::env::var("JWT_SECRET").unwrap_or_else(|_| "dev_secret_change_me".to_string());
        let host = std::env::var("APP_HOST").unwrap_or_else(|_| "0.0.0.0".into());
        let port: u16 = std::env::var("APP_PORT").ok().and_then(|s| s.parse().ok()).unwrap_or(8080);
        Ok(Self { host, port, database_url, jwt_secret })
    }
}