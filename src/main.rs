use std::sync::Arc;

use axum::{routing::get, Router};
use db::{sqlite::SqliteDatabase, DBInterface};

mod auth_handler;
mod db;

// Define the application state that will be shared across handlers
struct AppState {
    // db needs to be send and sync because it will be shared across multiple threads
    db: Box<dyn DBInterface + Send + Sync>,
}

#[tokio::main]
async fn main() {

    let shared_state = Arc::new(AppState {
        db: Box::new(SqliteDatabase::new("db.sqlite").expect("Failed to create database"))
    });

    let auth_router = auth_handler::auth_router(shared_state);

    let app = Router::new()
        .route("/hello", get(|| async { "Hello, World!" }))
        .nest("/auth", auth_router);

    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000")
        .await
        .expect("Failed to bind TCP listener");

    let server = axum::serve(listener, app);
        

    println!("Server running on http://localhost:3000");

    server.await.expect("Failed to start server");

}
