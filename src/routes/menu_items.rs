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
pub struct MenuItem {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub price: f64,
    pub category_id: String,
    pub created_by_id: String,
    pub is_active: i64,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

// Separate struct for JOIN queries
#[derive(Debug, sqlx::FromRow)]
pub struct MenuItemRow {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub price: f64,
    pub category_id: String,
    pub category_name: String, // joined from menu_categories
    pub created_by_id: String,
    pub is_active: i64,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

#[derive(Debug, Serialize)]
pub struct MenuItemResponse {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub price: f64,
    pub category_id: String,
    pub category_name: String,
    pub created_by_id: String,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<MenuItemRow> for MenuItemResponse {
    fn from(item: MenuItemRow) -> Self {
        Self {
            id: item.id,
            name: item.name,
            description: item.description,
            price: item.price,
            category_id: item.category_id,
            category_name: item.category_name,
            created_by_id: item.created_by_id,
            is_active: item.is_active != 0,
            created_at: item.created_at.and_utc(),
            updated_at: item.updated_at.and_utc(),
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct CreateMenuItem {
    pub name: String,
    pub description: Option<String>,
    pub price: f64,
    pub category_id: String,
    pub created_by_id: String,
}

pub async fn list(State(pool): State<SqlitePool>) -> Result<Json<Vec<MenuItemResponse>>> {
    let items = sqlx::query_as!(
        MenuItemRow,
        "SELECT
            mi.id,
            mi.name,
            mi.description,
            mi.price,
            mi.category_id,
            mi.is_active,
            mc.name AS category_name,
            mi.created_by_id,
            mi.created_at,
            mi.updated_at
         FROM menu_items mi
         JOIN menu_categories mc ON mc.id = mi.category_id"
    )
    .fetch_all(&pool)
    .await?
    .into_iter()
    .map(MenuItemResponse::from)
    .collect();

    Ok(Json(items))
}

pub async fn get(
    State(pool): State<SqlitePool>,
    Path(id): Path<String>,
) -> Result<Json<MenuItemResponse>> {
    let item = sqlx::query_as!(
        MenuItemRow,
        "SELECT
            mi.id,
            mi.name,
            mi.description,
            mi.price,
            mi.category_id,
            mi.is_active,
            mc.name AS category_name,
            mi.created_by_id,
            mi.created_at,
            mi.updated_at
         FROM menu_items mi
         JOIN menu_categories mc ON mc.id = mi.category_id
         WHERE mi.id = ?",
        id
    )
    .fetch_optional(&pool)
    .await?
    .ok_or(crate::error::AppError::NotFound)?;

    Ok(Json(MenuItemResponse::from(item)))
}

pub async fn create(
    State(pool): State<SqlitePool>,
    Json(payload): Json<CreateMenuItem>,
) -> Result<Json<MenuItemResponse>> {
    let id = Uuid::now_v7().to_string();

    sqlx::query!(
        "INSERT INTO menu_items (id, name, description, price, category_id, created_by_id)
         VALUES (?, ?, ?, ?, ?, ?)",
        id,
        payload.name,
        payload.description,
        payload.price,
        payload.category_id,
        payload.created_by_id,
    )
    .execute(&pool)
    .await?;

    let item = sqlx::query_as!(
        MenuItemRow,
        "SELECT
            mi.id,
            mi.name,
            mi.description,
            mi.price,
            mi.category_id,
            mi.is_active,
            mc.name AS category_name,
            mi.created_by_id,
            mi.created_at,
            mi.updated_at
         FROM menu_items mi
         JOIN menu_categories mc ON mc.id = mi.category_id
         WHERE mi.id = ?",
        id
    )
    .fetch_one(&pool)
    .await?;

    Ok(Json(MenuItemResponse::from(item)))
}
