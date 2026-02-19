use sqlx::sqlite::{SqlitePool, SqlitePoolOptions};
use std::env;

pub async fn connect() -> SqlitePool {
    let db_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");

    SqlitePoolOptions::new()
        .max_connections(5)
        .connect(&db_url)
        .await
        .expect("Failed to connect to SQLite")
}
