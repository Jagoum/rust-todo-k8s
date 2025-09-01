use anyhow::Context;
use sqlx::{postgres::PgPoolOptions, PgPool};

pub async fn connect_pool(db_url: &str) -> anyhow::Result<PgPool> {
    PgPoolOptions::new()
        .max_connections(10)
        .connect(db_url)
        .await
        .context("connect to Postgres")
}

pub async fn init_db(pool: &PgPool) -> anyhow::Result<()> {
    // users table
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
    .await?;

    // todos table
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
    .await?;

    Ok(())
}