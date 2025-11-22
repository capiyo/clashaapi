use axum::{
    routing::{get, post},
    Router,
};
use sqlx::MySqlPool;

pub fn routes() -> Router<MySqlPool> {
    Router::new()
        .route("/", post(crate::handlers::posts::create_post))
        .route("/", get(crate::handlers::posts::get_posts))
        .route("/:id", get(crate::handlers::posts::get_post_by_id))
}

pub fn upload_routes() -> Router<MySqlPool> {
    Router::new()
        .route("/uploads/:file_name", get(crate::handlers::upload::serve_image))
}