use crate::error::Result;
use axum::{
    Json,
    extract::{Path, State},
};
use chrono::{DateTime, NaiveDateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;
use uuid::Uuid;

#[derive(Debug, sqlx::FromRow)]
pub struct Order {
    pub id: String,
    pub status: String,
    pub table_number: i64,
    pub created_by_id: String,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

#[derive(Debug, Serialize)]
pub struct OrderResponse {
    pub id: String,
    pub status: String,
    pub table_number: i64,
    pub created_by_id: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<Order> for OrderResponse {
    fn from(o: Order) -> Self {
        Self {
            id: o.id,
            status: o.status,
            table_number: o.table_number,
            created_by_id: o.created_by_id,
            created_at: o.created_at.and_utc(),
            updated_at: o.updated_at.and_utc(),
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct CreateOrder {
    pub table_number: i64,
    pub created_by_id: String,
}

pub async fn list(State(pool): State<SqlitePool>) -> Result<Json<Vec<OrderResponse>>> {
    let orders = sqlx::query_as!(
        Order,
        "SELECT id, table_number, status, created_by_id, created_at, updated_at
         FROM orders"
    )
    .fetch_all(&pool)
    .await?
    .into_iter()
    .map(OrderResponse::from)
    .collect();

    Ok(Json(orders))
}

pub async fn get(
    State(pool): State<SqlitePool>,
    Path(id): Path<String>,
) -> Result<Json<OrderResponse>> {
    let order = sqlx::query_as!(
        Order,
        "SELECT id, table_number, status, created_by_id, created_at, updated_at
         FROM orders WHERE id = ?",
        id
    )
    .fetch_optional(&pool)
    .await?
    .ok_or(crate::error::AppError::NotFound)?;

    Ok(Json(OrderResponse::from(order)))
}

pub async fn create(
    State(pool): State<SqlitePool>,
    Json(payload): Json<CreateOrder>,
) -> Result<Json<OrderResponse>> {
    let id = Uuid::now_v7().to_string();

    let order = sqlx::query_as!(
        Order,
        "INSERT INTO orders (id, table_number, status, created_by_id)
         VALUES (?, ?, 'open', ?)
         RETURNING id, table_number, status, created_by_id, created_at, updated_at",
        id,
        payload.table_number,
        payload.created_by_id,
    )
    .fetch_one(&pool)
    .await?;

    Ok(Json(OrderResponse::from(order)))
}
