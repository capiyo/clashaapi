use axum::{
    extract::{State, Query},
    http::StatusCode,
    response::Json,
};
use serde::Deserialize;
use sqlx::MySqlPool;

use crate::models::game::{Game, CreateGame};

#[derive(Debug, Deserialize)]
pub struct GameQuery {
    pub status: Option<String>,
    pub league: Option<String>,
}

pub async fn get_games(
    State(pool): State<MySqlPool>,
    Query(query): Query<GameQuery>,
) -> Result<Json<Vec<Game>>, StatusCode> {
    println!("🔍 GET /api/games called - Starting query...");

    let games = if let Some(status) = query.status {
        println!("📋 Filtering by status: {}", status);
        sqlx::query_as::<_, Game>(
            "SELECT * FROM games WHERE status = ? AND (league = ? OR ? IS NULL) ORDER BY created_at DESC"
        )
            .bind(status)
            .bind(query.league.clone())
            .bind(query.league)
            .fetch_all(&pool)
            .await
    } else {
        println!("📋 No status filter");
        sqlx::query_as::<_, Game>(
            "SELECT * FROM games WHERE (league = ? OR ? IS NULL) ORDER BY created_at DESC"
        )
            .bind(query.league.clone())
            .bind(query.league)
            .fetch_all(&pool)
            .await
    }
        .map_err(|e| {
            eprintln!("❌ DATABASE ERROR: {:?}", e);
            eprintln!("💡 Error details: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    println!("✅ Successfully fetched {} games", games.len());
    Ok(Json(games))
}

pub async fn create_game(
    State(pool): State<MySqlPool>,
    Json(payload): Json<CreateGame>,
) -> Result<Json<Game>, StatusCode> {
    let game_id = sqlx::query!(
        r#"
        INSERT INTO games (
            home_team, away_team, league,
            home_win, away_win, draw, date, status
        ) VALUES (?, ?, ?, ?, ?, ?, ?, 'Upcoming')
        "#,
        payload.home_team,
        payload.away_team,
        payload.league,
        payload.home_win,
        payload.away_win,
        payload.draw,
        payload.date
    )
        .execute(&pool)
        .await
        .map_err(|e| {
            eprintln!("❌ INSERT ERROR: {:?}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?
        .last_insert_id();

    let game = sqlx::query_as::<_, Game>(
        "SELECT * FROM games WHERE id = ?"
    )
        .bind(game_id)
        .fetch_one(&pool)
        .await
        .map_err(|e| {
            eprintln!("❌ SELECT ERROR: {:?}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    println!("✅ Successfully created game: {} vs {}", game.home_team, game.away_team);
    Ok(Json(game))
}