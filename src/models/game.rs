use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use chrono::{DateTime, Utc};

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct Game {
    pub id: i32,
    pub home_team: String,
    pub away_team: String,
    pub league: String,
    pub home_win: String,        // Changed to match frontend
    pub away_win: String,        // Changed to match frontend
    pub draw: String,            // Changed to match frontend
    pub date: String,            // Changed to match frontend
    pub status: String,          // Changed to String for MySQL compatibility
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateGame {
    pub home_team: String,
    pub away_team: String,
    pub league: String,
    pub home_win: String,        // Changed to match frontend
    pub away_win: String,        // Changed to match frontend
    pub draw: String,            // Changed to match frontend
    pub date: String,            // Changed to match frontend
}