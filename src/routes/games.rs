use axum::{
    routing::{get, post},
    Router,
};
use sqlx::MySqlPool;

use crate::handlers::games::{get_games, create_game};

pub fn routes() -> Router<MySqlPool> {
    Router::new()
        .route("/", get(get_games))
        .route("/post", post(create_game))
}