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
pub struct MenuCategory {
    pub id:            String,
    pub name:          String,
    pub description:   Option<String>,
    pub sort_order:    i64,
    pub is_active:  i64,
    pub created_by_id: String,
    pub created_at:    NaiveDateTime,
    pub updated_at:    NaiveDateTime,
}

#[derive(Debug, Serialize)]
pub struct MenuCategoryResponse {
    pub id:            String,
    pub name:          String,
    pub description:   Option<String>,
    pub sort_order:    i64,
    pub is_active:     bool,
    pub created_by_id: String,
    pub created_at:    DateTime<Utc>,
    pub updated_at:    DateTime<Utc>,
}

impl From<MenuCategory> for MenuCategoryResponse {
    fn from(c: MenuCategory) -> Self {
        Self {
            id:            c.id,
            name:          c.name,
            description:   c.description,
            sort_order:    c.sort_order,
            is_active:     c.is_active != 0,
            created_by_id: c.created_by_id,
            created_at:    c.created_at.and_utc(),
            updated_at:    c.updated_at.and_utc(),
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct CreateMenuCategory {
    pub name:          String,
    pub description:   Option<String>,
    pub sort_order:    Option<i64>,
    pub created_by_id: String,
}

pub async fn list(State(pool): State<SqlitePool>) -> Result<Json<Vec<MenuCategoryResponse>>> {
    let categories = sqlx::query_as!(
        MenuCategory,
        "SELECT id, name, description, sort_order, is_active, created_by_id, created_at, updated_at
         FROM menu_categories
         ORDER BY sort_order ASC"
    )
    .fetch_all(&pool)
    .await?
    .into_iter()
    .map(MenuCategoryResponse::from)
    .collect();

    Ok(Json(categories))
}

pub async fn get(
    State(pool): State<SqlitePool>,
    Path(id): Path<String>,
) -> Result<Json<MenuCategoryResponse>> {
    let category = sqlx::query_as!(
        MenuCategory,
        "SELECT id, name, description, sort_order, is_active, created_by_id, created_at, updated_at
         FROM menu_categories WHERE id = ?",
        id
    )
    .fetch_optional(&pool)
    .await?
    .ok_or(crate::error::AppError::NotFound)?;

    Ok(Json(MenuCategoryResponse::from(category)))
}

pub async fn create(
    State(pool): State<SqlitePool>,
    Json(payload): Json<CreateMenuCategory>,
) -> Result<Json<MenuCategoryResponse>> {
    let id = Uuid::now_v7().to_string();
    let sort_order = payload.sort_order.unwrap_or(0);

    let category = sqlx::query_as!(
        MenuCategory,
        "INSERT INTO menu_categories (id, name, description, sort_order, created_by_id)
         VALUES (?, ?, ?, ?, ?)
         RETURNING id, name, description, sort_order, is_active, created_by_id, created_at, updated_at",
        id,
        payload.name,
        payload.description,
        sort_order,
        payload.created_by_id,
    )
    .fetch_one(&pool)
    .await?;

    Ok(Json(MenuCategoryResponse::from(category)))
}