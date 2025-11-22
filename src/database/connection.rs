use sqlx::mysql::MySqlPoolOptions;
use sqlx::MySqlPool;
use std::time::Duration;

pub async fn get_db_pool() -> MySqlPool {
    let database_url = std::env::var("DATABASE_URL")
        .expect("DATABASE_URL must be set in .env file");

    MySqlPoolOptions::new()
        .max_connections(10)
        .acquire_timeout(Duration::from_secs(5))
        .connect(&database_url)
        .await
        .expect("Failed to create database pool")
}