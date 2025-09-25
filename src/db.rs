/* src/db.rs */

use crate::error::{PathmapError, Result};
use sqlx::{
    Row, SqlitePool,
    sqlite::{SqliteConnectOptions, SqlitePoolOptions},
};
use std::path::Path;

pub async fn connect(db_path: &Path) -> Result<SqlitePool> {
    // This logic remains crucial. SQLite will not create the parent directory.
    if let Some(parent) = db_path.parent() {
        if !parent.exists() {
            // Ensure the base directory for our databases exists.
            std::fs::create_dir_all(parent)?;
        }
    }

    // Be more explicit with connection options to ensure the database file is created.
    let connection_options = SqliteConnectOptions::new()
        .filename(db_path)
        .create_if_missing(true); // Explicitly tell sqlx to create the DB file

    // Use `connect_with` to apply our explicit options.
    let pool = SqlitePoolOptions::new()
        .max_connections(5)
        .connect_with(connection_options)
        .await?;

    // Create table if not exists using a dynamic query
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS kv_store (
            key TEXT PRIMARY KEY NOT NULL,
            value BLOB NOT NULL
        )
        "#,
    )
    .execute(&pool)
    .await?;

    Ok(pool)
}

pub async fn get(pool: &SqlitePool, key: &str) -> Result<Vec<u8>> {
    let row = sqlx::query("SELECT value FROM kv_store WHERE key = ?")
        .bind(key)
        .fetch_optional(pool)
        .await?;

    row.map(|r| r.get("value"))
        .ok_or_else(|| PathmapError::ValueNotFound(key.to_string()))
}

pub async fn set(pool: &SqlitePool, key: &str, value: &[u8]) -> Result<()> {
    sqlx::query("INSERT INTO kv_store (key, value) VALUES (?, ?)")
        .bind(key)
        .bind(value)
        .execute(pool)
        .await?;
    Ok(())
}

pub async fn exists(pool: &SqlitePool, key: &str) -> Result<bool> {
    let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM kv_store WHERE key LIKE ?")
        .bind(format!("{}%", key))
        .fetch_one(pool)
        .await?;
    Ok(count > 0)
}

pub async fn delete(pool: &SqlitePool, key: &str) -> Result<()> {
    sqlx::query("DELETE FROM kv_store WHERE key = ?")
        .bind(key)
        .execute(pool)
        .await?;
    Ok(())
}

pub async fn overwrite(pool: &SqlitePool, key: &str, value: &[u8]) -> Result<()> {
    sqlx::query("INSERT OR REPLACE INTO kv_store (key, value) VALUES (?, ?)")
        .bind(key)
        .bind(value)
        .execute(pool)
        .await?;
    Ok(())
}

pub async fn vacuum(pool: &SqlitePool) -> Result<()> {
    sqlx::query("VACUUM").execute(pool).await?;
    Ok(())
}

/// Lists all keys starting with a given prefix.
pub async fn list_keys(pool: &SqlitePool, prefix: &str) -> Result<Vec<String>> {
    let query_pattern = format!("{}%", prefix);
    let rows = sqlx::query("SELECT key FROM kv_store WHERE key LIKE ?")
        .bind(query_pattern)
        .fetch_all(pool)
        .await?;

    let keys = rows.into_iter().map(|row| row.get("key")).collect();
    Ok(keys)
}
