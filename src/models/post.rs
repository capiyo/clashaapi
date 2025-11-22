use serde::{Deserialize, Serialize};
use chrono::{NaiveDateTime, DateTime, Utc};
use sqlx::FromRow;

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct Post {
    pub id: String,
    pub user_id: String,
    pub user_name: String,
    pub caption: String,
    pub image_url: String,
    pub image_path: String,
    pub created_at: NaiveDateTime,  // Use NaiveDateTime directly
    pub updated_at: NaiveDateTime,  // Use NaiveDateTime directlyrgo
}

#[derive(Debug, Serialize)]
pub struct PostResponse {
    pub id: String,
    pub user_id: String,
    pub user_name: String,
    pub caption: String,
    pub image_url: String,
    #[serde(with = "chrono::serde::ts_seconds")]  // Serialize as timestamp
    pub created_at: DateTime<Utc>,
}

impl From<Post> for PostResponse {
    fn from(post: Post) -> Self {
        Self {
            id: post.id,
            user_id: post.user_id,
            user_name: post.user_name,
            caption: post.caption,
            image_url: post.image_url,
            created_at: DateTime::from_utc(post.created_at, Utc),  // Convert here
        }
    }
}