use std::sync::Arc;

use axum::{routing::get, Router};
use crypt::crypt_provider::CryptProviders;
use db::{sqlite::SqliteDatabase, DBInterface};
use log::info;

mod auth_handler;
mod db;
mod crypt;
mod data_handler;

// Define the application state that will be shared across handlers
struct AppState<DB: DBInterface + Send + Sync> {
    // db needs to be send and sync because it will be shared across multiple threads
    // this can be any struct that implements DBInterface
    db: Box<DB>,
    crypt_provider: CryptProviders
}

#[tokio::main]
async fn main() {
    env_logger::init();

    let shared_state = Arc::new(AppState {
        db: Box::new(SqliteDatabase::new("data/db.sqlite").expect("Failed to create database")),
        crypt_provider: CryptProviders::SimpleCryptProv
    });

    let auth_router = auth_handler::auth_router(shared_state.clone());
    let data_router = data_handler::data_router(shared_state.clone());

    let app = Router::new()
        .route("/hello", get(|| async { "Hello, World!" }))
        .nest("/auth", auth_router)
        .nest("/data", data_router);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000")
        .await
        .expect("Failed to bind TCP listener");

    let server = axum::serve(listener, app);
        
    info!("Server running on http://localhost:3000");

    server.await.expect("Failed to start server");

}
