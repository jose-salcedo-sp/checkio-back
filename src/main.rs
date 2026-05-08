mod auth;
mod config;
mod db;
mod error;
mod routes;
mod utils;
use axum::http::{HeaderValue, Method};
use tower_http::cors::{Any, CorsLayer};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "checkio_back=debug,axum=debug,tower_http=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .with(tracing_subscriber::fmt::layer().json())
        .init();

    let config = config::Config::from_env();
    let pool = db::create_pool(&config.database_url).await;

    let cors = CorsLayer::new()
        .allow_origin(config.client_origin.parse::<HeaderValue>().unwrap())
        .allow_methods([Method::GET, Method::POST, Method::PUT, Method::PATCH])
        .allow_headers(Any);

    let app = routes::app_router(pool).layer(cors);
    let addr = format!("{}:{}", config.host, config.port);
    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();

    println!("Listening on {addr}");
    axum::serve(listener, app).await.unwrap();
}
