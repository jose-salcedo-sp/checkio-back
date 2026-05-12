use crate::error::{AppError, Result};
use axum::{
    Json,
    extract::{Path, State},
};
use chrono::{DateTime, NaiveDateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;
use uuid::Uuid;

#[derive(Debug, sqlx::FromRow)]
pub struct Table {
    pub id: String,
    pub label: String,
    pub sort_order: i64,
    pub is_active: i64,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

#[derive(Debug, Serialize)]
pub struct TableResponse {
    pub id: String,
    pub label: String,
    pub sort_order: i64,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<Table> for TableResponse {
    fn from(t: Table) -> Self {
        Self {
            id: t.id,
            label: t.label,
            sort_order: t.sort_order,
            is_active: t.is_active != 0,
            created_at: t.created_at.and_utc(),
            updated_at: t.updated_at.and_utc(),
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct CreateTable {
    pub label: String,
    pub sort_order: Option<i64>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateTable {
    pub label: Option<String>,
    pub sort_order: Option<i64>,
    pub is_active: Option<bool>,
}

pub async fn list(State(pool): State<SqlitePool>) -> Result<Json<Vec<TableResponse>>> {
    let rows = sqlx::query_as!(
        Table,
        "SELECT id, label, sort_order, is_active, created_at, updated_at
         FROM tables
         ORDER BY sort_order ASC, label ASC"
    )
    .fetch_all(&pool)
    .await?
    .into_iter()
    .map(TableResponse::from)
    .collect();

    Ok(Json(rows))
}

pub async fn get(
    State(pool): State<SqlitePool>,
    Path(id): Path<String>,
) -> Result<Json<TableResponse>> {
    let row = sqlx::query_as!(
        Table,
        "SELECT id, label, sort_order, is_active, created_at, updated_at
         FROM tables WHERE id = ?",
        id
    )
    .fetch_optional(&pool)
    .await?
    .ok_or(AppError::NotFound)?;

    Ok(Json(TableResponse::from(row)))
}

pub async fn create(
    State(pool): State<SqlitePool>,
    Json(payload): Json<CreateTable>,
) -> Result<Json<TableResponse>> {
    let id = Uuid::now_v7().to_string();
    let sort_order = payload.sort_order.unwrap_or(0);

    let row = sqlx::query_as!(
        Table,
        "INSERT INTO tables (id, label, sort_order)
         VALUES (?, ?, ?)
         RETURNING id, label, sort_order, is_active, created_at, updated_at",
        id,
        payload.label,
        sort_order,
    )
    .fetch_one(&pool)
    .await
    .map_err(unique_label_err)?;

    Ok(Json(TableResponse::from(row)))
}

pub async fn update(
    State(pool): State<SqlitePool>,
    Path(id): Path<String>,
    Json(patch): Json<UpdateTable>,
) -> Result<Json<TableResponse>> {
    let existing = sqlx::query_as!(
        Table,
        "SELECT id, label, sort_order, is_active, created_at, updated_at
         FROM tables WHERE id = ?",
        id
    )
    .fetch_optional(&pool)
    .await?
    .ok_or(AppError::NotFound)?;

    let label = patch.label.unwrap_or(existing.label);
    let sort_order = patch.sort_order.unwrap_or(existing.sort_order);
    let is_active = patch
        .is_active
        .map(|b| if b { 1i64 } else { 0 })
        .unwrap_or(existing.is_active);

    let row = sqlx::query_as!(
        Table,
        "UPDATE tables SET label = ?, sort_order = ?, is_active = ?, updated_at = CURRENT_TIMESTAMP
         WHERE id = ?
         RETURNING id, label, sort_order, is_active, created_at, updated_at",
        label,
        sort_order,
        is_active,
        id,
    )
    .fetch_one(&pool)
    .await
    .map_err(unique_label_err)?;

    Ok(Json(TableResponse::from(row)))
}

fn unique_label_err(e: sqlx::Error) -> AppError {
    if let Some(db) = e.as_database_error() {
        if db.message().to_ascii_lowercase().contains("unique") {
            return AppError::BadRequest("duplicate table label".into());
        }
    }
    AppError::Database(e)
}
