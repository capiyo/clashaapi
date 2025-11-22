use axum::{
    response::Json,
};
use uuid::Uuid;
use chrono::Utc;
use std::path::Path;
use tokio::fs;
use serde_json::json;

use crate::errors::{AppError, Result};
use crate::models::post::{Post, PostResponse};
use sqlx::MySqlPool;
use axum_extra::extract::Multipart;

const MAX_FILE_SIZE: u64 = 10 * 1024 * 1024; // 10MB
const ALLOWED_EXTENSIONS: [&str; 4] = ["jpg", "jpeg", "png", "gif"];

pub async fn create_post(
    axum::extract::State(pool): axum::extract::State<MySqlPool>,
    mut multipart: Multipart,
) -> Result<Json<serde_json::Value>> {
    let mut caption = String::new();
    let mut user_id = String::new();
    let mut user_name = String::new();
    let mut image_data = None;
    let mut file_extension = None;

    // Process multipart form data
    while let Some(field) = multipart.next_field().await.map_err(|e| AppError::Multipart(e.to_string()))? {
        let field_name = field.name().unwrap_or("").to_string();

        match field_name.as_str() {
            "caption" => {
                caption = field.text().await.map_err(|e| AppError::Multipart(e.to_string()))?;
            }
            "userId" => {
                user_id = field.text().await.map_err(|e| AppError::Multipart(e.to_string()))?;
            }
            "userName" => {
                user_name = field.text().await.map_err(|e| AppError::Multipart(e.to_string()))?;
            }
            "image" => {
                let file_name = field.file_name().unwrap_or("image").to_string();
                let data = field.bytes().await.map_err(|e| AppError::Multipart(e.to_string()))?;

                // Validate file size
                if data.len() as u64 > MAX_FILE_SIZE {
                    return Err(AppError::ImageTooLarge);
                }

                // Validate file type
                let ext = Path::new(&file_name)
                    .extension()
                    .and_then(|ext| ext.to_str())
                    .unwrap_or("")
                    .to_lowercase();

                if !ALLOWED_EXTENSIONS.contains(&ext.as_str()) {
                    return Err(AppError::InvalidImageFormat);
                }

                file_extension = Some(ext);
                image_data = Some(data);
            }
            _ => {}
        }
    }

    // Validate required fields
    if user_id.is_empty() || user_name.is_empty() {
        return Err(AppError::InvalidUserData);
    }

    let image_data = image_data.ok_or(AppError::NoImageProvided)?;
    let file_extension = file_extension.ok_or(AppError::InvalidImageFormat)?;

    // Create uploads directory if it doesn't exist
    fs::create_dir_all("uploads/images").await.map_err(AppError::Io)?;

    // Generate unique filename
    let file_name = format!("{}.{}", Uuid::new_v4(), file_extension);
    let file_path = format!("uploads/images/{}", file_name);
    let image_url = format!("/api/uploads/{}", file_name);

    // Save image to filesystem
    fs::write(&file_path, &image_data).await.map_err(AppError::Io)?;

    // Create post in database
    let post_id = Uuid::new_v4();
    let now = Utc::now();

    // Updated SQL query - removed user_email
    sqlx::query!(
        r#"
        INSERT INTO posts (id, user_id, user_name, caption, image_url, image_path, created_at, updated_at)
        VALUES (?, ?, ?, ?, ?, ?, ?, ?)
        "#,
        post_id.to_string(),
        user_id,
        user_name,
        caption,
        image_url,
        file_path,
        now,
        now
    )
        .execute(&pool)
        .await
        .map_err(AppError::Database)?;

    Ok(Json(json!({
        "success": true,
        "message": "Post created successfully",
        "post": {
            "id": post_id,
            "image_url": image_url,
            "caption": caption,
            "user_name": user_name
        }
    })))
}

pub async fn get_posts(
    axum::extract::State(pool): axum::extract::State<MySqlPool>
) -> Result<Json<serde_json::Value>> {
    let posts = sqlx::query_as!(
        Post,
        r#"
        SELECT id, user_id, user_name, caption, image_url, image_path, created_at, updated_at
        FROM posts
        ORDER BY created_at DESC
        "#
    )
        .fetch_all(&pool)
        .await
        .map_err(AppError::Database)?;

    let post_responses: Vec<PostResponse> = posts.into_iter().map(PostResponse::from).collect();

    Ok(Json(json!({
        "success": true,
        "posts": post_responses
    })))
}

pub async fn get_post_by_id(
    axum::extract::State(pool): axum::extract::State<MySqlPool>,
    axum::extract::Path(post_id): axum::extract::Path<Uuid>,
) -> Result<Json<serde_json::Value>> {
    let post = sqlx::query_as!(
        Post,
        r#"
        SELECT id, user_id, user_name, caption, image_url, image_path, created_at, updated_at
        FROM posts
        WHERE id = ?
        "#,
        post_id.to_string()
    )
        .fetch_optional(&pool)
        .await
        .map_err(AppError::Database)?;

    match post {
        Some(post) => Ok(Json(json!({
            "success": true,
            "post": PostResponse::from(post)
        }))),
        None => Err(AppError::PostNotFound),
    }
}

// Get posts by user ID
pub async fn get_posts_by_user(
    axum::extract::State(pool): axum::extract::State<MySqlPool>,
    axum::extract::Path(user_id): axum::extract::Path<String>,
) -> Result<Json<serde_json::Value>> {
    let posts = sqlx::query_as!(
        Post,
        r#"
        SELECT id, user_id, user_name, caption, image_url, image_path, created_at, updated_at
        FROM posts
        WHERE user_id = ?
        ORDER BY created_at DESC
        "#,
        user_id
    )
        .fetch_all(&pool)
        .await
        .map_err(AppError::Database)?;

    let post_responses: Vec<PostResponse> = posts.into_iter().map(PostResponse::from).collect();

    Ok(Json(json!({
        "success": true,
        "posts": post_responses,
        "count": post_responses.len()
    })))
}

// Delete post
pub async fn delete_post(
    axum::extract::State(pool): axum::extract::State<MySqlPool>,
    axum::extract::Path(post_id): axum::extract::Path<Uuid>,
) -> Result<Json<serde_json::Value>> {
    // First get the post to find the image path
    let post = sqlx::query_as!(
        Post,
        r#"
        SELECT id, user_id, user_name, caption, image_url, image_path, created_at, updated_at
        FROM posts
        WHERE id = ?
        "#,
        post_id.to_string()
    )
        .fetch_optional(&pool)
        .await
        .map_err(AppError::Database)?;

    match post {
        Some(post) => {
            // Delete the image file from filesystem
            if let Err(e) = fs::remove_file(&post.image_path).await {
                tracing::warn!("Failed to delete image file {}: {}", post.image_path, e);
            }

            // Delete the post from database
            sqlx::query!(
                r#"
                DELETE FROM posts
                WHERE id = ?
                "#,
                post_id.to_string()
            )
                .execute(&pool)
                .await
                .map_err(AppError::Database)?;

            Ok(Json(json!({
                "success": true,
                "message": "Post deleted successfully"
            })))
        }
        None => Err(AppError::PostNotFound),
    }
}

// Update post caption
pub async fn update_post_caption(
    axum::extract::State(pool): axum::extract::State<MySqlPool>,
    axum::extract::Path(post_id): axum::extract::Path<Uuid>,
    axum::extract::Json(payload): axum::extract::Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>> {
    let new_caption = payload.get("caption")
        .and_then(|c| c.as_str())
        .ok_or(AppError::InvalidUserData)?;

    let result = sqlx::query!(
        r#"
        UPDATE posts 
        SET caption = ?, updated_at = ?
        WHERE id = ?
        "#,
        new_caption,
        Utc::now(),
        post_id.to_string()
    )
        .execute(&pool)
        .await
        .map_err(AppError::Database)?;

    if result.rows_affected() == 0 {
        return Err(AppError::PostNotFound);
    }

    Ok(Json(json!({
        "success": true,
        "message": "Post caption updated successfully"
    })))
}