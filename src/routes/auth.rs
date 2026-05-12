use crate::{
    auth::create_token,
    config,
    error::{AppError, Result},
    utils::passwords::verify_password,
};
use axum::{Json, extract::State};
use serde::Deserialize;
use sqlx::SqlitePool;

#[derive(Deserialize)]
pub struct LoginUser {
    email: String,
    password: String,
}

pub async fn login(
    State(pool): State<SqlitePool>,
    Json(payload): Json<LoginUser>,
) -> Result<String> {
    let row = sqlx::query!(
        r#"SELECT u.id, u.password_hash, r.slug as role
           FROM users u
           INNER JOIN roles r ON r.id = u.role_id
           WHERE u.email = ?
           LIMIT 1"#,
        payload.email
    )
    .fetch_optional(&pool)
    .await?
    .ok_or(AppError::Unauthorized)?;

    if !verify_password(payload.password, row.password_hash)? {
        return Err(AppError::Unauthorized);
    }
    let config = config::Config::from_env();
    let token = create_token(&row.id, &row.role, &config.jwt_secret)?;
    Ok(token)
}
