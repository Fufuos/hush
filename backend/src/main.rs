use std::net::SocketAddr;

mod database;
mod handlers;
mod routes;

#[tokio::main]
async fn main() {
    dotenvy::from_path(std::env::current_dir().unwrap().join(".env")).ok();
    if std::env::var("DATABASE_URL").is_err() {
        dotenvy::dotenv().ok();
    }

    // Get the DB URL, defaulting to a local file if not set
    let db_url = std::env::var("DATABASE_URL").unwrap_or_else(|_| "sqlite://hush.db".to_string());

    // Initialize the DB pool using our bulletproof function
    let pool = database::init_db(&db_url).await;

    let app = routes::create_router(pool);

    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    println!("Hush server listening on http://{}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
