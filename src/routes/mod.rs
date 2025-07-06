// src/routes/mod.rs
pub mod auth;
pub mod game;

use axum::{
    routing::{get, post},
    Router,
    middleware,
    // Extension, // Удален: не используется напрямую в create_router
};
use tokio::sync::broadcast;
use std::sync::Arc;

use crate::routes::game::GameMessage;

pub type SharedGameState = Arc<broadcast::Sender<GameMessage>>;

pub fn create_router() -> Router {
    Router::new()
        .route("/register", post(auth::register))
        .route("/login", post(auth::login))
        // Применяем auth_middleware только к маршруту /ws
        .route("/ws", get(game::websocket_handler).layer(middleware::from_fn(auth::auth_middleware)))
}