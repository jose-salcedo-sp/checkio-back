use axum::{Router, middleware, routing::{get, post}};
use sqlx::SqlitePool;
use tower_http::trace::TraceLayer;

use crate::utils::auth::AuthUser;

mod menu_item_categories;
mod menu_items;
mod orders;
mod tables;
mod users;
mod auth;

pub fn app_router(pool: SqlitePool) -> Router {
    Router::new()
        .nest("/api/auth", auth_routes())
        .nest("/api", protected_routes())
        .layer(TraceLayer::new_for_http())
        .with_state(pool)
}

fn protected_routes() -> Router<SqlitePool> {
    Router::new()
        .nest("/users", user_routes())
        .nest("/menu_items", menu_item_routes())
        .nest("/menu_item_categories", menu_item_category_routes())
        .nest("/orders", order_routes())
        .nest("/tables", table_routes())
        .route_layer(middleware::from_extractor::<AuthUser>())
}

fn auth_routes() -> Router<SqlitePool> {
    Router::new()
        .route("/login", post(auth::login))
}

fn user_routes() -> Router<SqlitePool> {
    Router::new()
        .route("/", get(users::list).post(users::create))
        .route("/{id}", get(users::get))
}

fn menu_item_routes() -> Router<SqlitePool> {
    Router::new()
        .route("/", get(menu_items::list).post(menu_items::create))
        .route("/{id}", get(menu_items::get))
}

fn order_routes() -> Router<SqlitePool> {
    Router::new()
        .route("/", get(orders::list).post(orders::create))
        .route("/{id}", get(orders::get))
}

fn table_routes() -> Router<SqlitePool> {
    Router::new()
        .route("/", get(tables::list).post(tables::create))
        .route("/{id}", get(tables::get).patch(tables::update))
}

fn menu_item_category_routes() -> Router<SqlitePool> {
    Router::new()
        .route("/", get(menu_item_categories::list).post(menu_item_categories::create))
        .route("/{id}", get(menu_item_categories::get))
}