use axum::{
    extract::{State, Query},
    http::StatusCode,
    response::Json,
};
use serde::Deserialize;
use sqlx::MySqlPool;
use chrono::Utc;

use crate::models::pledges::{Pledge, CreatePledge, PledgeQuery};

#[derive(Debug, Deserialize)]
pub struct PledgeStatsQuery {
    pub home_team: Option<String>,
    pub away_team: Option<String>,
}

// Get all pledges with optional filtering
pub async fn get_pledges(
    State(pool): State<MySqlPool>,
    Query(query): Query<PledgeQuery>,
) -> Result<Json<Vec<Pledge>>, StatusCode> {
    println!("🔍 GET /api/pledges called - Starting query...");

    let mut sql = "SELECT * FROM pledges WHERE 1=1".to_string();
    let mut params: Vec<String> = Vec::new();

    if let Some(username) = &query.username {
        sql.push_str(" AND username = ?");
        params.push(username.clone());
    }

    if let Some(phone) = &query.phone {
        sql.push_str(" AND phone = ?");
        params.push(phone.clone());
    }

    if let Some(home_team) = &query.home_team {
        sql.push_str(" AND home_team = ?");
        params.push(home_team.clone());
    }

    if let Some(away_team) = &query.away_team {
        sql.push_str(" AND away_team = ?");
        params.push(away_team.clone());
    }

    sql.push_str(" ORDER BY created_at DESC");

    println!("📋 Executing query: {}", sql);

    let mut query_builder = sqlx::query_as::<_, Pledge>(&sql);

    for param in params {
        query_builder = query_builder.bind(param);
    }

    let pledges = query_builder
        .fetch_all(&pool)
        .await
        .map_err(|e| {
            eprintln!("❌ DATABASE ERROR: {:?}", e);
            eprintln!("💡 Error details: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    println!("✅ Successfully fetched {} pledges", pledges.len());
    Ok(Json(pledges))
}

// Create a new pledge
pub async fn create_pledge(
    State(pool): State<MySqlPool>,
    Json(payload): Json<CreatePledge>,
) -> Result<Json<Pledge>, StatusCode> {
    println!("🎯 Creating new pledge for user: {}", payload.username);

    // Validate required fields
    if payload.username.is_empty() || payload.phone.is_empty() || payload.selection.is_empty() {
        return Err(StatusCode::BAD_REQUEST);
    }

    if payload.amount <= 0.0 {
        return Err(StatusCode::BAD_REQUEST);
    }

    let result = sqlx::query(
        r#"
        INSERT INTO pledges (
            username, phone, selection, amount, time, fan,
            home_team, away_team, created_at, updated_at
        ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, NOW(), NOW())
        "#
    )
        .bind(&payload.username)
        .bind(&payload.phone)
        .bind(&payload.selection)
        .bind(payload.amount)
        .bind(Utc::now())
        .bind(&payload.fan)
        .bind(&payload.home_team)
        .bind(&payload.away_team)
        .execute(&pool)
        .await
        .map_err(|e| {
            eprintln!("❌ INSERT ERROR: {:?}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    let pledge_id = result.last_insert_id();

    let pledge = sqlx::query_as::<_, Pledge>(
        "SELECT * FROM pledges WHERE id = ?"
    )
        .bind(pledge_id)
        .fetch_one(&pool)
        .await
        .map_err(|e| {
            eprintln!("❌ SELECT ERROR: {:?}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    println!("✅ Successfully created pledge for user: {} - Amount: ₿{}", payload.username, payload.amount);
    Ok(Json(pledge))
}

// Get pledge statistics for a specific match
pub async fn get_pledge_stats(
    State(pool): State<MySqlPool>,
    Query(query): Query<PledgeStatsQuery>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    println!("📊 Getting pledge statistics...");

    let (home_team, away_team) = match (&query.home_team, &query.away_team) {
        (Some(home), Some(away)) => (home, away),
        _ => return Err(StatusCode::BAD_REQUEST),
    };

    // Get total pledges count
    let total_pledges: (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM pledges WHERE home_team = ? AND away_team = ?"
    )
        .bind(home_team)
        .bind(away_team)
        .fetch_one(&pool)
        .await
        .map_err(|e| {
            eprintln!("❌ COUNT ERROR: {:?}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    // Get total amount pledged
    let total_amount: (Option<f64>,) = sqlx::query_as(
        "SELECT SUM(amount) FROM pledges WHERE home_team = ? AND away_team = ?"
    )
        .bind(home_team)
        .bind(away_team)
        .fetch_one(&pool)
        .await
        .map_err(|e| {
            eprintln!("❌ SUM ERROR: {:?}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    // Get selection breakdown
    let home_pledges: (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM pledges WHERE home_team = ? AND away_team = ? AND selection = 'home_team'"
    )
        .bind(home_team)
        .bind(away_team)
        .fetch_one(&pool)
        .await
        .map_err(|e| {
            eprintln!("❌ HOME PLEDGES ERROR: {:?}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    let away_pledges: (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM pledges WHERE home_team = ? AND away_team = ? AND selection = 'away_team'"
    )
        .bind(home_team)
        .bind(away_team)
        .fetch_one(&pool)
        .await
        .map_err(|e| {
            eprintln!("❌ AWAY PLEDGES ERROR: {:?}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    let draw_pledges: (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM pledges WHERE home_team = ? AND away_team = ? AND selection = 'draw'"
    )
        .bind(home_team)
        .bind(away_team)
        .fetch_one(&pool)
        .await
        .map_err(|e| {
            eprintln!("❌ DRAW PLEDGES ERROR: {:?}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    let stats = serde_json::json!({
        "total_pledges": total_pledges.0,
        "total_amount": total_amount.0.unwrap_or(0.0),
        "selection_breakdown": {
            "home_team": home_pledges.0,
            "away_team": away_pledges.0,
            "draw": draw_pledges.0
        },
        "match": {
            "home_team": home_team,
            "away_team": away_team
        }
    });

    println!("✅ Successfully fetched pledge statistics");
    Ok(Json(stats))
}

// Get user's pledging history
pub async fn get_user_pledges(
    State(pool): State<MySqlPool>,
    Query(query): Query<PledgeQuery>,
) -> Result<Json<Vec<Pledge>>, StatusCode> {
    println!("👤 Getting user pledges...");

    let username = query.username.ok_or(StatusCode::BAD_REQUEST)?;

    let pledges = sqlx::query_as::<_, Pledge>(
        "SELECT * FROM pledges WHERE username = ? ORDER BY created_at DESC"
    )
        .bind(username)
        .fetch_all(&pool)
        .await
        .map_err(|e| {
            eprintln!("❌ USER PLEDGES ERROR: {:?}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    println!("✅ Successfully fetched {} pledges for user", pledges.len());
    Ok(Json(pledges))
}

// Get recent pledges (for social proof)
pub async fn get_recent_pledges(
    State(pool): State<MySqlPool>,
) -> Result<Json<Vec<Pledge>>, StatusCode> {
    println!("🕒 Getting recent pledges...");

    let pledges = sqlx::query_as::<_, Pledge>(
        "SELECT * FROM pledges ORDER BY created_at DESC LIMIT 10"
    )
        .fetch_all(&pool)
        .await
        .map_err(|e| {
            eprintln!("❌ RECENT PLEDGES ERROR: {:?}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    println!("✅ Successfully fetched {} recent pledges", pledges.len());
    Ok(Json(pledges))
}