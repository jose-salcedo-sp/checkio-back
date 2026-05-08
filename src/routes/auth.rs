use crate::{
    auth::create_token,
    config,
    error::{AppError, Result},
    routes::users::User,
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
    let user = sqlx::query_as!(
        User,
        "SELECT * FROM users WHERE email = ? LIMIT 1",
        payload.email
    )
    .fetch_optional(&pool)
    .await?
    .ok_or(AppError::Unauthorized)?;

    if !verify_password(payload.password, user.password_hash)? {
        return Err(AppError::Unauthorized);
    }
    let config = config::Config::from_env();
    let token = create_token(&user.id, &user.role, &config.jwt_secret)?;
    Ok(token)
}
