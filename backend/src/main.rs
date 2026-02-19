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

    let pool = database::connect().await;

    let app = routes::create_router(pool);

    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    println!("Hush server listening on http://{}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
