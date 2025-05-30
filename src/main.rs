use std::{sync::Arc, vec};

use axum::{routing::get, Router};
use crypt::crypt_provider::CryptProviders;
use db::{sqlite::SqliteDatabase, DBInterface, Course};
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
        db: Box::new(SqliteDatabase::new("db.sqlite").expect("Failed to create database")),
        crypt_provider: CryptProviders::SimpleCryptProv
    });

    // test code
    shared_state.db.create_table_for_type::<Course>().unwrap();
    //shared_state.db.new_entry::<Course>(vec![SQLWhereValue::Blob(vec![4,5,8])]).unwrap();

    let entries = shared_state.db.select_entries::<Course>(select_fields! {name: vec![4,5,8]}).unwrap();
    println!("{:?}", entries);
    // test code end

    let auth_router = auth_handler::auth_router(shared_state.clone());
    let data_router = data_handler::data_router(shared_state.clone());

    let app = Router::new()
        .route("/hello", get(|| async { "Hello, World!" }))
        .nest("/auth", auth_router)
        .nest("/data", data_router);

    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000")
        .await
        .expect("Failed to bind TCP listener");

    let server = axum::serve(listener, app);
        
    info!("Server running on http://localhost:3000");

    server.await.expect("Failed to start server");

}
