use sqlx::PgPool;
use crate::models::user::User;

pub async fn find_user_by_login(pool: &PgPool, login: &str) -> Result<Option<User>, sqlx::Error> {
    sqlx::query_as::<_, User>("SELECT id, login, hashed_password FROM users WHERE login = $1")
        .bind(login)
        .fetch_optional(pool)
        .await
}