use tracing_subscriber;
use axum::{
    extract::State,
    http::{HeaderMap, StatusCode, Method},
    response::Json,
    routing::{get, post},
    Router,
};
use axum_extra::extract::Multipart;
use tower_http::cors::{Any, CorsLayer};
use std::net::SocketAddr;

mod routes;
mod models;
mod handlers;
mod middleware;
mod database;
mod errors;

use routes::{auth, games, posts};
use database::connection::get_db_pool;

#[tokio::main]
async fn main() {
    // Load environment variables
    dotenvy::dotenv().ok();

    // Initialize tracing
    tracing_subscriber::fmt::init();

    // Create uploads directory if it doesn't exist
    if let Err(e) = tokio::fs::create_dir_all("uploads/images").await {
        tracing::warn!("Failed to create uploads directory: {}", e);
    }

    // Initialize database pool
    let pool = get_db_pool().await;

    // CORS configuration
    let cors = CorsLayer::new()
        .allow_methods([Method::GET, Method::POST, Method::PUT, Method::DELETE])
        .allow_headers(Any)
        .allow_origin(Any);

    // Build application
    let app = Router::new()
        .route("/", get(|| async { "Peer-to-Peer Betting API" }))
        .nest("/api/auth", auth::routes())
        .nest("/api/games", games::routes())
        .nest("/api/posts", posts::routes())
        .nest("/api", posts::upload_routes()) // Add upload routes for serving images
        .layer(cors)
        .with_state(pool);

    let addr = SocketAddr::from(([0, 0, 0, 0], 3000));
    tracing::info!("Server running on {}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}