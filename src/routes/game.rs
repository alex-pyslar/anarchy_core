use axum::{
    extract::{WebSocketUpgrade, ws::{Message, WebSocket}},
    response::IntoResponse,
    Extension
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::broadcast;

use crate::state::AppState;
use crate::routes::auth::Claims;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayerPositionUpdate {
    pub user_id: i32,
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "payload")]
pub enum GameMessage {
    PlayerPosition(PlayerPositionUpdate),
    PlayerDisconnected { user_id: i32 },
    InitialPlayers(Vec<PlayerPositionUpdate>),
    PlayerLogout { user_id: i32 },
}

pub type SharedGameState = Arc<broadcast::Sender<GameMessage>>;

pub async fn websocket_handler(
    ws: WebSocketUpgrade,
    Extension(app_state): Extension<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
) -> impl IntoResponse {
    ws.on_upgrade(move |socket| handle_socket(socket, app_state, claims))
}

async fn handle_socket(mut socket: WebSocket, app_state: Arc<AppState>, claims: Claims) {
    let mut game_state_rx = app_state.game_state_tx.subscribe();

    let current_user_id: i32 = claims.sub.parse().unwrap_or_else(|_| {
        eprintln!("Failed to parse user_id from claims.sub: {}", claims.sub);
        panic!("Invalid user ID in claims!");
    });

    println!("DEBUG: Client {} connected via WebSocket", current_user_id);

    // --- Начальная загрузка позиции игрока ---
    let mut initial_player_pos = sqlx::query_as!(
        PlayerPositionUpdate,
        "SELECT user_id, x, y, z FROM players WHERE user_id = $1",
        current_user_id
    )
        .fetch_optional(&app_state.pool)
        .await
        .unwrap_or_else(|e| {
            eprintln!("Error fetching player position from DB for user {}: {:?}", current_user_id, e);
            None
        })
        .unwrap_or_else(|| {
            PlayerPositionUpdate {
                user_id: current_user_id,
                x: 0.0,
                y: 0.0,
                z: 0.0,
            }
        });

    let mut active_players_map = app_state.active_player_positions.lock().await;
    active_players_map.insert(current_user_id, initial_player_pos.clone());
    drop(active_players_map);

    // Отправляем данные начальных игроков новому подключенному клиенту
    let active_players_vec: Vec<PlayerPositionUpdate> = app_state.active_player_positions.lock().await.values().cloned().collect();

    if let Ok(serialized_msg) = serde_json::to_string(&GameMessage::InitialPlayers(active_players_vec)) {
        if socket.send(Message::Text(serialized_msg)).await.is_err() {
            eprintln!("Failed to send initial active players to client {}.", current_user_id);
        } else {
            println!("DEBUG: Sent InitialPlayers to client {}", current_user_id);
        }
    }

    let mut logout_processed = false; // Флаг для отслеживания обработки PlayerLogout

    // Основной цикл для приема и широковещания сообщений
    loop {
        tokio::select! {
            // Принимаем сообщения от этого клиента
            Some(msg_result) = socket.recv() => {
                match msg_result {
                    Ok(msg) => {
                        if let Message::Text(text) = msg {
                            if let Ok(game_msg) = serde_json::from_str::<GameMessage>(&text) {
                                match game_msg {
                                    GameMessage::PlayerPosition(mut player_update) => {
                                        // Проверяем и перезаписываем user_id для безопасности
                                        if player_update.user_id != current_user_id {
                                            eprintln!("Client {} tried to send position for user_id {} (mismatch with authenticated user_id {}). Overwriting.",
                                                current_user_id, player_update.user_id, current_user_id);
                                            player_update.user_id = current_user_id;
                                        }

                                        // Сохраняем/обновляем позицию в БД
                                        if let Err(e) = sqlx::query!(
                                            "INSERT INTO players (user_id, x, y, z) VALUES ($1, $2, $3, $4)
                                             ON CONFLICT (user_id) DO UPDATE SET x = $2, y = $3, z = $4",
                                            current_user_id,
                                            player_update.x,
                                            player_update.y,
                                            player_update.z
                                        )
                                        .execute(&app_state.pool)
                                        .await {
                                            eprintln!("Error updating player position in DB for user {}: {:?}", current_user_id, e);
                                        }

                                        // Обновляем позицию в in-memory HashMap
                                        let mut active_players_map = app_state.active_player_positions.lock().await;
                                        active_players_map.insert(current_user_id, player_update.clone());
                                        drop(active_players_map);

                                        // Широковещательно отправляем новое состояние
                                        let broadcast_player_update = PlayerPositionUpdate {
                                            user_id: current_user_id,
                                            x: player_update.x,
                                            y: player_update.y,
                                            z: player_update.z,
                                        };
                                        if let Err(e) = app_state.game_state_tx.send(GameMessage::PlayerPosition(broadcast_player_update)) {
                                            eprintln!("Error broadcasting PlayerPosition for user {}: {:?}", current_user_id, e);
                                        } else {
                                            println!("DEBUG: Broadcasted PlayerPosition for user {}", current_user_id);
                                        }
                                    },
                                    GameMessage::PlayerLogout { user_id } => {
                                        if user_id == current_user_id {
                                            println!("DEBUG: Received PlayerLogout for user {}", current_user_id);
                                            logout_processed = true; // Устанавливаем флаг
                                            // Отправляем PlayerDisconnected сразу
                                            let mut active_players_map = app_state.active_player_positions.lock().await;
                                            active_players_map.remove(&current_user_id);
                                            drop(active_players_map);
                                            println!("DEBUG: Number of subscribers for user {}: {}", current_user_id, app_state.game_state_tx.receiver_count());
                                            if let Err(e) = app_state.game_state_tx.send(GameMessage::PlayerDisconnected { user_id: current_user_id }) {
                                                eprintln!("Error broadcasting PlayerDisconnected for user {}: {:?}", current_user_id, e);
                                            } else {
                                                println!("DEBUG: Broadcasted PlayerDisconnected for user {} on PlayerLogout", current_user_id);
                                            }
                                            // Задержка для гарантии доставки сообщения
                                            tokio::time::sleep(tokio::time::Duration::from_millis(200)).await;
                                            break; // Выходим из цикла после отправки
                                        } else {
                                            eprintln!("Client {} sent PlayerLogout for user_id {} (mismatch). Ignoring.", current_user_id, user_id);
                                        }
                                    },
                                    _ => {
                                        eprintln!("Received unexpected GameMessage type from client {}: {:?}", current_user_id, game_msg);
                                    }
                                }
                            } else {
                                eprintln!("Received unparseable text as GameMessage from client {}: {}", current_user_id, text);
                            }
                        } else if matches!(msg, Message::Close(_)) {
                            println!("DEBUG: Client {} sent close message.", current_user_id);
                            break;
                        } else {
                            println!("DEBUG: Received other message type from client {}: {:?}", current_user_id, msg);
                        }
                    },
                    Err(e) => {
                        eprintln!("WebSocket receive error for client {}: {:?}", current_user_id, e);
                        break;
                    }
                }
            }
            // Принимаем сообщения из канала широковещания (для других клиентов)
            Ok(broadcast_msg) = game_state_rx.recv() => {
                // InitialPlayers предназначено только для нового клиента
                if matches!(broadcast_msg, GameMessage::InitialPlayers(_)) {
                    continue;
                }
                // Сообщение об отключении не отправляем обратно отключившемуся клиенту
                if let GameMessage::PlayerDisconnected { user_id: disconnected_id } = &broadcast_msg {
                    if current_user_id == *disconnected_id {
                        continue;
                    }
                }

                if let Ok(serialized_msg) = serde_json::to_string(&broadcast_msg) {
                    if socket.send(Message::Text(serialized_msg)).await.is_err() {
                        eprintln!("Failed to send broadcast message to client {}.", current_user_id);
                        break;
                    } else {
                        println!("DEBUG: Sent broadcast message to client {}: {:?}", current_user_id, broadcast_msg);
                    }
                }
            }
        }
    }

    // --- Обработка отключения: отправка сообщения об отключении и удаление из активных ---
    println!("DEBUG: Client {} disconnected.", current_user_id);

    // Удаляем игрока из in-memory HashMap активных игроков, если еще не удален
    let mut active_players_map = app_state.active_player_positions.lock().await;
    if active_players_map.contains_key(&current_user_id) {
        active_players_map.remove(&current_user_id);
        println!("DEBUG: Removed user {} from active_players_map", current_user_id);
    } else {
        println!("DEBUG: User {} was not in active_players_map", current_user_id);
    }
    drop(active_players_map);

    // Отправляем PlayerDisconnected только если не отправляли при PlayerLogout
    if !logout_processed {
        println!("DEBUG: Number of subscribers for user {}: {}", current_user_id, app_state.game_state_tx.receiver_count());
        if let Err(e) = app_state.game_state_tx.send(GameMessage::PlayerDisconnected { user_id: current_user_id }) {
            eprintln!("Error broadcasting PlayerDisconnected for user {}: {:?}", current_user_id, e);
        } else {
            println!("DEBUG: Broadcasted PlayerDisconnected for user {} on disconnect", current_user_id);
        }
    } else {
        println!("DEBUG: Skipped broadcasting PlayerDisconnected for user {} as it was sent on PlayerLogout", current_user_id);
    }
}