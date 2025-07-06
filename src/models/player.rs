use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize)]
pub struct Player {
    pub user_id: i32,
    pub x: f32,
    pub y: f32,
    pub z: f32,
}