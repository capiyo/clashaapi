use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use chrono::{DateTime, Utc};

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct Pledge {
    pub id: i32,
    pub username: String,
    pub phone: String,
    pub selection: String,        // home_team, away_team, or draw
    pub amount: f64,
    pub time: DateTime<Utc>,
    pub fan: String,              // fan type/level
    pub home_team: String,
    pub away_team: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreatePledge {
    pub username: String,
    pub phone: String,
    pub selection: String,
    pub amount: f64,
    pub fan: String,
    pub home_team: String,
    pub away_team: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PledgeQuery {
    pub username: Option<String>,
    pub phone: Option<String>,
    pub home_team: Option<String>,
    pub away_team: Option<String>,
    pub status: Option<String>,
}