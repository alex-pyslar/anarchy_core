// src/routes/auth.rs
use axum::{
    extract::{Json},
    response::{IntoResponse, Response},
    http::StatusCode,
    Extension,
    body::Body, // Добавлено: для Request<Body>
};
use axum::middleware::Next;
use axum::http::Request;
use serde::{Deserialize, Serialize};
use bcrypt::{hash, verify, DEFAULT_COST};
use jsonwebtoken::{encode, decode, DecodingKey, Validation, Header, EncodingKey};
use chrono::Utc;
use std::sync::Arc;

use crate::{models::user::User, config::Config, state::AppState};

#[derive(Serialize, Deserialize)]
pub struct RegisterRequest {
    login: String,
    password: String,
}

#[derive(Serialize, Deserialize)]
pub struct LoginResponse {
    token: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)] // Добавлены Clone и Debug
pub struct Claims {
    pub sub: String,
    pub exp: usize,
}

#[derive(Serialize)]
struct ErrorResponse {
    message: String,
}

// JWT Authentication Middleware
pub async fn auth_middleware(
    Extension(app_state): Extension<Arc<AppState>>,
    mut req: Request<Body>,
    next: Next,
) -> Result<Response, StatusCode> {
    let config = Config::from_env().unwrap();
    let jwt_secret = config.jwt_secret.as_bytes();

    let token = req.headers()
        .get(axum::http::header::AUTHORIZATION)
        .and_then(|header| header.to_str().ok())
        .and_then(|header_str| header_str.strip_prefix("Bearer ").map(|s| s.to_string()));

    let token_string = if let Some(t) = token {
        t
    } else {
        return Err(StatusCode::UNAUTHORIZED);
    };

    let claims = decode::<Claims>(
        &token_string,
        &DecodingKey::from_secret(jwt_secret),
        &Validation::default(),
    );

    match claims {
        Ok(token_data) => {
            req.extensions_mut().insert(token_data.claims);
            Ok(next.run(req).await)
        },
        Err(e) => {
            eprintln!("JWT validation failed: {:?}", e);
            Err(StatusCode::UNAUTHORIZED)
        }
    }
}

pub async fn register(
    Extension(app_state): Extension<Arc<AppState>>,
    Json(payload): Json<RegisterRequest>,
) -> impl IntoResponse {
    let hashed_password = hash(&payload.password, DEFAULT_COST).unwrap();
    match sqlx::query!(
        "INSERT INTO users (login, hashed_password) VALUES ($1, $2)",
        payload.login,
        hashed_password
    )
        .execute(&app_state.pool)
        .await
    {
        Ok(_) => "Registered successfully".into_response(),
        Err(e) => {
            let error_msg = format!("Registration failed: {}", e);
            (StatusCode::BAD_REQUEST, Json(ErrorResponse { message: error_msg })).into_response()
        },
    }
}

pub async fn login(
    Extension(app_state): Extension<Arc<AppState>>,
    Json(payload): Json<RegisterRequest>,
) -> Result<Json<LoginResponse>, Response> {
    let user = sqlx::query_as::<_, User>("SELECT id, login, hashed_password FROM users WHERE login = $1")
        .bind(payload.login)
        .fetch_one(&app_state.pool)
        .await
        .map_err(|e| {
            eprintln!("Login DB fetch error: {:?}", e);
            (StatusCode::UNAUTHORIZED, Json(ErrorResponse { message: "Invalid credentials".to_string() })).into_response()
        })?;

    if verify(&payload.password, &user.hashed_password).unwrap_or(false) {
        let config = Config::from_env().unwrap();
        let claims = Claims {
            sub: user.id.to_string(),
            exp: (Utc::now() + chrono::Duration::hours(24)).timestamp() as usize,
        };
        let token = encode(
            &Header::default(),
            &claims,
            &EncodingKey::from_secret(config.jwt_secret.as_bytes()),
        )
            .map_err(|e| {
                eprintln!("Login JWT encoding error: {:?}", e);
                (StatusCode::INTERNAL_SERVER_ERROR, Json(ErrorResponse { message: "Failed to generate token".to_string() })).into_response()
            })?;
        Ok(Json(LoginResponse { token }))
    } else {
        Err((StatusCode::UNAUTHORIZED, Json(ErrorResponse { message: "Invalid credentials".to_string() })).into_response())
    }
}