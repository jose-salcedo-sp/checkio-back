use crate::error::Result;
use crate::utils::passwords::hash_password;
use axum::{
    Json,
    extract::{Path, State},
};
use chrono::{DateTime, NaiveDateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;
use uuid::Uuid;

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct User {
    pub id: String,
    pub name: String,
    pub email: String,
    pub role: String,
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

impl From<User> for UserResponse {
    fn from(value: User) -> Self {
        Self {
            id: value.id,
            name: value.name,
            email: value.email,
            role: value.role,
            is_active: value.is_active != 0,
            created_at: value.created_at.and_utc(),
            updated_at: value.updated_at.and_utc(),
        }
    }
}

#[derive(Deserialize)]
pub struct CreateUser {
    pub name: String,
    pub email: String,
    pub role: String,
    pub password: String,
}

pub async fn list(State(pool): State<SqlitePool>) -> Result<Json<Vec<UserResponse>>> {
    let users = sqlx::query_as!(
        User,
        "SELECT id, name, email, role, is_active, created_at, updated_at, password_hash FROM users"
    )
    .fetch_all(&pool)
    .await?
    .into_iter()
    .map(UserResponse::from)
    .collect();

    Ok(Json(users))
}

pub async fn get(State(pool): State<SqlitePool>, Path(id): Path<String>) -> Result<Json<User>> {
    let user = sqlx::query_as!(User, "SELECT * FROM users WHERE id = ?", id)
        .fetch_optional(&pool)
        .await?
        .ok_or(crate::error::AppError::NotFound)?;
    Ok(Json(user))
}

pub async fn create(
    State(pool): State<SqlitePool>,
    Json(payload): Json<CreateUser>,
) -> Result<Json<UserResponse>> {
    let id = Uuid::now_v7().to_string();
    let password_hash = hash_password(payload.password)?;

    let user = sqlx::query_as!(
        User,
        "INSERT INTO users (id, name, email, password_hash, role, is_active)
         VALUES (?, ?, ?, ?, ?, ?)
         RETURNING id, name, email, role, password_hash, is_active, created_at, updated_at",
        id,
        payload.name,
        payload.email,
        password_hash,
        payload.role,
        true,
    )
    .fetch_one(&pool)
    .await?
    .into();

    Ok(Json(user))
}
