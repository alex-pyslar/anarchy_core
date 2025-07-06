use sqlx::PgPool;
use crate::models::player::Player;

pub async fn update_player_position(pool: &PgPool, player: &Player) -> Result<(), sqlx::Error> {
    sqlx::query!(
        "INSERT INTO players (user_id, x, y, z) VALUES ($1, $2, $3, $4)
         ON CONFLICT (user_id) DO UPDATE SET x = $2, y = $3, z = $4",
        player.user_id,
        player.x,
        player.y,
        player.z
    )
        .execute(pool)
        .await?;
    Ok(())
}