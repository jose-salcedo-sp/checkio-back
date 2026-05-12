use crate::error::{AppError, Result};
use crate::utils::passwords::hash_password;
use axum::{
    Json,
    extract::{Path, State},
};
use chrono::{DateTime, NaiveDateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;
use uuid::Uuid;

/// Row matching the `users` table only (`role_id`, no role slug). Use with
/// `query_as!(User, "SELECT … FROM users …")` when you are not joining `roles`.
#[allow(dead_code)]
#[derive(Debug, sqlx::FromRow)]
pub struct User {
    pub id: String,
    pub name: String,
    pub email: String,
    pub role_id: String,
    pub password_hash: String,
    pub is_active: i64,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

#[derive(Serialize)]
pub struct UserResponse {
    pub id: String,
    pub name: String,
    pub email: String,
    pub role: String,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Deserialize)]
pub struct CreateUser {
    pub name: String,
    pub email: String,
    pub role: String,
    pub password: String,
}

fn user_response_from_join(
    id: String,
    name: String,
    email: String,
    role: String,
    is_active: i64,
    created_at: NaiveDateTime,
    updated_at: NaiveDateTime,
) -> UserResponse {
    UserResponse {
        id,
        name,
        email,
        role,
        is_active: is_active != 0,
        created_at: created_at.and_utc(),
        updated_at: updated_at.and_utc(),
    }
}

pub async fn list(State(pool): State<SqlitePool>) -> Result<Json<Vec<UserResponse>>> {
    let rows = sqlx::query!(
        r#"SELECT u.id, u.name, u.email, r.slug as role, u.is_active, u.created_at, u.updated_at
           FROM users u
           INNER JOIN roles r ON r.id = u.role_id
           ORDER BY u.name ASC"#
    )
    .fetch_all(&pool)
    .await?;

    let users = rows
        .into_iter()
        .map(|r| {
            user_response_from_join(
                r.id,
                r.name,
                r.email,
                r.role,
                r.is_active,
                r.created_at,
                r.updated_at,
            )
        })
        .collect();

    Ok(Json(users))
}

pub async fn get(
    State(pool): State<SqlitePool>,
    Path(id): Path<String>,
) -> Result<Json<UserResponse>> {
    let r = sqlx::query!(
        r#"SELECT u.id, u.name, u.email, r.slug as role, u.is_active, u.created_at, u.updated_at
           FROM users u
           INNER JOIN roles r ON r.id = u.role_id
           WHERE u.id = ?"#,
        id
    )
    .fetch_optional(&pool)
    .await?
    .ok_or(crate::error::AppError::NotFound)?;

    Ok(Json(user_response_from_join(
        r.id,
        r.name,
        r.email,
        r.role,
        r.is_active,
        r.created_at,
        r.updated_at,
    )))
}

pub async fn create(
    State(pool): State<SqlitePool>,
    Json(payload): Json<CreateUser>,
) -> Result<Json<UserResponse>> {
    let role_id: String = sqlx::query_scalar!(
        "SELECT id FROM roles WHERE slug = ?",
        payload.role
    )
    .fetch_optional(&pool)
    .await?
    .ok_or_else(|| AppError::BadRequest("unknown role".into()))?;

    let id = Uuid::now_v7().to_string();
    let password_hash = hash_password(payload.password)?;

    sqlx::query!(
        "INSERT INTO users (id, name, email, password_hash, role_id, is_active)
         VALUES (?, ?, ?, ?, ?, ?)",
        id,
        payload.name,
        payload.email,
        password_hash,
        role_id,
        true,
    )
    .execute(&pool)
    .await?;

    let r = sqlx::query!(
        r#"SELECT u.id, u.name, u.email, r.slug as role, u.is_active, u.created_at, u.updated_at
           FROM users u
           INNER JOIN roles r ON r.id = u.role_id
           WHERE u.id = ?"#,
        id
    )
    .fetch_one(&pool)
    .await?;

    Ok(Json(user_response_from_join(
        r.id,
        r.name,
        r.email,
        r.role,
        r.is_active,
        r.created_at,
        r.updated_at,
    )))
}
