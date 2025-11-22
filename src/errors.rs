use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum AppError {
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),

    #[error("Multipart error: {0}")]
    Multipart(String),  // Changed from multer::Error to String

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Invalid image format")]
    InvalidImageFormat,

    #[error("Image too large")]
    ImageTooLarge,

    #[error("No image provided")]
    NoImageProvided,

    #[error("Invalid user data")]
    InvalidUserData,

    #[error("Post not found")]
    PostNotFound,
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, error_message) = match self {
            AppError::Database(_) => (StatusCode::INTERNAL_SERVER_ERROR, "Database error"),
            AppError::Multipart(_) => (StatusCode::BAD_REQUEST, "Invalid multipart data"),
            AppError::Io(_) => (StatusCode::INTERNAL_SERVER_ERROR, "IO error"),
            AppError::InvalidImageFormat => (StatusCode::BAD_REQUEST, "Invalid image format"),
            AppError::ImageTooLarge => (StatusCode::BAD_REQUEST, "Image too large"),
            AppError::NoImageProvided => (StatusCode::BAD_REQUEST, "No image provided"),
            AppError::InvalidUserData => (StatusCode::BAD_REQUEST, "Invalid user data"),
            AppError::PostNotFound => (StatusCode::NOT_FOUND, "Post not found"),
        };

        let body = Json(json!({
            "error": error_message,
            "message": self.to_string(),
        }));

        (status, body).into_response()
    }
}

pub type Result<T> = std::result::Result<T, AppError>;