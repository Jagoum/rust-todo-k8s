use anyhow::Context;

pub struct Config {
    pub host: String,
    pub port: u16,
    pub database_url: String,
    pub jwt_secret: String,
}

impl Config {
    pub fn from_env() -> anyhow::Result<Self> {
        let database_url = std::env::var("DATABASE_URL").context("DATABASE_URL env var is required")?;
        let jwt_secret = std::env::var("JWT_SECRET").unwrap_or_else(|_| "dev_secret_change_me".to_string());
        let host = std::env::var("APP_HOST").unwrap_or_else(|_| "0.0.0.0".into());
        let port: u16 = std::env::var("APP_PORT").ok().and_then(|s| s.parse().ok()).unwrap_or(8080);
        Ok(Self { host, port, database_url, jwt_secret })
    }
}