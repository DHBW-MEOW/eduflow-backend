use std::sync::Arc;

use axum::{routing::get, Router};
use crypt::crypt_provider::CryptProviders;
use db::{sqlite::SqliteDatabase, DBInterface};
use log::info;

mod auth_handler;
mod db;
mod crypt;

// Define the application state that will be shared across handlers
struct AppState {
    // db needs to be send and sync because it will be shared across multiple threads
    db: Box<dyn DBInterface + Send + Sync>,
    crypt_provider: CryptProviders
}

#[tokio::main]
async fn main() {
    env_logger::init();



    let shared_state = Arc::new(AppState {
        db: Box::new(SqliteDatabase::new("db.sqlite").expect("Failed to create database")),
        crypt_provider: CryptProviders::SimpleCryptProv
    });

    let auth_router = auth_handler::auth_router(shared_state);

    let app = Router::new()
        .route("/hello", get(|| async { "Hello, World!" }))
        .nest("/auth", auth_router);

    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000")
        .await
        .expect("Failed to bind TCP listener");

    let server = axum::serve(listener, app);
        

    info!("Server running on http://localhost:3000");

    // crypt tests
    //let crypt_prov = SimpleCryptProv {};
    //let key = b"hallo";

    //let secret_text = CryptString::encrypt(&"test".to_string(), key, &crypt_prov);
    //let secret_i = CryptI32::encrypt(&12, key, &crypt_prov);

    //shared_state.db.new_dummy("testdummy", &secret_i, &secret_text).unwrap();

    //let db_dumm = shared_state.db.get_dummy(1).unwrap();

    //let dumm_text = db_dumm.secret_text.decrypt(key, &crypt_prov).unwrap();

    //debug!("Decrypted from DB: {}", dumm_text);


    server.await.expect("Failed to start server");

}
