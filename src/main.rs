use std::{env, sync::Arc};

use axum::{
    Router,
    http::{
        Method,
        header::{AUTHORIZATION, CONTENT_TYPE},
    },
    routing::get,
};
use crypt::crypt_provider::CryptProviders;
use db::{DBInterface, sqlite::SqliteDatabase};
use log::info;
use tower_http::cors::CorsLayer;

mod auth_handler;
mod crypt;
mod data_handler;
mod db;

// Define the application state that will be shared across handlers
struct AppState<DB: DBInterface + Send + Sync> {
    // db needs to be send and sync because it will be shared across multiple threads
    // this can be any struct that implements DBInterface
    db: Box<DB>,
    crypt_provider: CryptProviders,
}

#[tokio::main]
async fn main() {
    env_logger::init();

    let shared_state = Arc::new(AppState {
        db: Box::new(SqliteDatabase::new("data/db.sqlite").expect("Failed to create database")),
        crypt_provider: CryptProviders::SimpleCryptProv,
    });

    let origins = [
        env::var("FRONTEND_CORS_URL")
            .unwrap_or("http://localhost:5173".to_string())
            .parse()
            .unwrap(), // get frontend url from env or use default
    ];
    let cors = CorsLayer::new()
        .allow_origin(origins)
        .allow_methods([Method::GET, Method::POST, Method::DELETE])
        .allow_headers([AUTHORIZATION, CONTENT_TYPE])
        .allow_credentials(true);

    let auth_router = auth_handler::auth_router(shared_state.clone());
    let data_router = data_handler::data_router(shared_state.clone());

    let app = Router::new()
        .route("/hello", get(|| async { "Hello, World!" }))
        .nest("/auth", auth_router)
        .nest("/data", data_router)
        .layer(cors);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000")
        .await
        .expect("Failed to bind TCP listener");

    let server = axum::serve(listener, app);

    info!("Server running on http://localhost:3000");

    server.await.expect("Failed to start server");
}
