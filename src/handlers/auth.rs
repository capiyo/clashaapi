use axum::{
    extract::State,
    http::StatusCode,
    response::Json,
};
use bcrypt::{hash, verify, DEFAULT_COST};
use jsonwebtoken::{encode, EncodingKey, Header};
use chrono::Utc;
use sqlx::MySqlPool;

use crate::models::user::{
    User, CreateUser, LoginUser, LoginWithPhone, UserResponse, AuthResponse, Claims
};

pub async fn register(
    State(pool): State<MySqlPool>,
    Json(payload): Json<CreateUser>,
) -> Result<Json<AuthResponse>, StatusCode> {
    // Check if user exists by username or phone
    let existing_user = sqlx::query_as::<_, User>(
        "SELECT * FROM users WHERE username = ? OR phone = ?"
    )
        .bind(&payload.username)
        .bind(&payload.phone)
        .fetch_optional(&pool)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    if existing_user.is_some() {
        return Err(StatusCode::CONFLICT);
    }

    // Hash password
    let password_hash = hash(&payload.password, DEFAULT_COST)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // Insert user
    let result = sqlx::query(
        "INSERT INTO users (username, phone, password_hash, balance, created_at, updated_at) VALUES (?, ?, ?, 0.0, NOW(), NOW())"
    )
        .bind(&payload.username)
        .bind(&payload.phone)
        .bind(&password_hash)
        .execute(&pool)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let user_id = result.last_insert_id();

    // Generate JWT token
    let user_response = UserResponse {
        id: user_id as i32,
        username: payload.username.clone(),
        phone: payload.phone.clone(),
        balance: 0.0,
    };

    let claims = Claims {
        sub: user_id as i32,
        username: payload.username,
        phone: payload.phone,
        exp: (Utc::now().timestamp() + 86400) as usize, // 24 hours
    };

    let secret = std::env::var("JWT_SECRET").unwrap_or_else(|_| "secret".to_string());
    let token = encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret.as_ref()),
    )
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(AuthResponse {
        user: user_response,
        token,
    }))
}

pub async fn login(
    State(pool): State<MySqlPool>,
    Json(payload): Json<LoginUser>,
) -> Result<Json<AuthResponse>, StatusCode> {
    let user = sqlx::query_as::<_, User>(
        "SELECT * FROM users WHERE username = ?"
    )
        .bind(&payload.username)
        .fetch_optional(&pool)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::UNAUTHORIZED)?;

    let valid = verify(&payload.password, &user.password_hash)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    if !valid {
        return Err(StatusCode::UNAUTHORIZED);
    }

    // Generate JWT token
    let user_response = UserResponse {
        id: user.id,
        username: user.username.clone(),
        phone: user.phone.clone(),
        balance: user.balance,
    };

    let claims = Claims {
        sub: user.id,
        username: user.username,
        phone: user.phone,
        exp: (Utc::now().timestamp() + 86400) as usize,
    };

    let secret = std::env::var("JWT_SECRET").unwrap_or_else(|_| "secret".to_string());
    let token = encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret.as_ref()),
    )
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(AuthResponse {
        user: user_response,
        token,
    }))
}

pub async fn login_with_phone(
    State(pool): State<MySqlPool>,
    Json(payload): Json<LoginWithPhone>,
) -> Result<Json<AuthResponse>, StatusCode> {
    let user = sqlx::query_as::<_, User>(
        "SELECT * FROM users WHERE phone = ?"
    )
        .bind(&payload.phone)
        .fetch_optional(&pool)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::UNAUTHORIZED)?;

    let valid = verify(&payload.password, &user.password_hash)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    if !valid {
        return Err(StatusCode::UNAUTHORIZED);
    }

    // Generate JWT token
    let user_response = UserResponse {
        id: user.id,
        username: user.username.clone(),
        phone: user.phone.clone(),
        balance: user.balance,
    };

    let claims = Claims {
        sub: user.id,
        username: user.username,
        phone: user.phone,
        exp: (Utc::now().timestamp() + 86400) as usize,
    };

    let secret = std::env::var("JWT_SECRET").unwrap_or_else(|_| "secret".to_string());
    let token = encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret.as_ref()),
    )
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(AuthResponse {
        user: user_response,
        token,
    }))
}