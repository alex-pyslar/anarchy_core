// src/main.rs
mod config;
mod routes;
mod models;
mod state;

use axum::{Router, serve, Extension};
use sqlx::postgres::PgPoolOptions;
use std::net::SocketAddr;
use {config::Config, routes::create_router};
use tokio::net::TcpListener;
use tokio::sync::broadcast;
use std::sync::Arc;
use tokio::sync::Mutex;
use std::collections::HashMap;

use crate::state::AppState;
use crate::routes::game::GameMessage;

#[tokio::main]
async fn main() {
    // Загрузка конфигурации
    let config = Config::from_env().expect("Failed to load config");

    // Подключение к PostgreSQL
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&config.database_url)
        .await
        .expect("Failed to connect to DB");

    // Инициализация канала широковещания для сообщений о состоянии игры
    let (game_state_tx, _) = broadcast::channel::<GameMessage>(128); // Увеличен размер канала до 128

    // Инициализация HashMap для активных игроков
    let active_player_positions = Arc::new(Mutex::new(HashMap::new()));

    // Создаем экземпляр AppState
    let app_state = Arc::new(AppState {
        pool,
        game_state_tx: Arc::new(game_state_tx),
        active_player_positions,
    });

    // Создание роутера и передача AppState как Extension
    let app = Router::new()
        .nest("/api", create_router())
        .layer(Extension(app_state));

    // Запуск сервера с помощью axum::serve
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    println!("Server running on {}", addr);
    let listener = TcpListener::bind(addr).await.unwrap();
    serve(listener, app.into_make_service()).await.unwrap();
}