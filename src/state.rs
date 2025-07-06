// src/state.rs
use sqlx::PgPool;
use tokio::sync::broadcast;
use std::sync::Arc;
use tokio::sync::Mutex;
use std::collections::HashMap;

// Импортируем типы из game.rs, так как они используются в AppState
// Путь зависит от того, где GameMessage и PlayerPositionUpdate определены.
// Если они в game.rs, то так:
use crate::routes::game::{GameMessage, PlayerPositionUpdate};

// Структура для общего состояния приложения
pub struct AppState {
    pub pool: PgPool,
    pub game_state_tx: Arc<broadcast::Sender<GameMessage>>,
    // HashMap для отслеживания текущих позиций ТОЛЬКО активных игроков
    pub active_player_positions: Arc<Mutex<HashMap<i32, PlayerPositionUpdate>>>,
}