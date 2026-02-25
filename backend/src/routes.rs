use crate::handlers::{create_secret, delete_secret, get_secret, list_secrets, update_secret};
use axum::{Extension, Router, routing::get};
use sqlx::SqlitePool;

pub fn create_router(pool: SqlitePool) -> Router {
    Router::new()
        .route("/secrets", get(list_secrets).post(create_secret))
        // Chain the GET and DELETE handlers together
        // Update this line:
        .route(
            "/secrets/{name}",
            get(get_secret).delete(delete_secret).put(update_secret),
        )
        .layer(Extension(pool))
}
