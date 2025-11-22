use axum::{
    routing::{post},
    Router,
};
use sqlx::MySqlPool;

use crate::handlers::auth::{register, login};

pub fn routes() -> Router<MySqlPool> {
    Router::new()
        .route("/register", post(register))
        .route("/login", post(login))
}