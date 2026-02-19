use axum::{
    Extension, Router,
    routing::{get, post},
};
use sqlx::SqlitePool;

use crate::handlers::{create_secret, get_secret};

pub fn create_router(pool: SqlitePool) -> Router {
    Router::new()
        .route("/secrets", post(create_secret))
        .route("/secrets/{name}", get(get_secret))
        // Share the DB pool with all routes
        .layer(Extension(pool))
}
