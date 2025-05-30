use std::sync::Arc;

use axum::{routing::get, Router};
use crypt::crypt_provider::CryptProviders;
use db::{sql_helper::SQLWhereValue, sqlite::SqliteDatabase, DBInterface, ModuleDB, TestDummy};
use log::info;
use db::sql_helper::SQLGenerate;

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

    // test code
    let map = select_fields! {
        name: "test",
        id: 1,
    };

    info!("{:?}", map);

    info!("{}", TestDummy::get_db_table_create());
    info!("{}", TestDummy::get_db_insert());
    info!("{}", TestDummy::get_db_select(map.keys().collect()));
    // test code end

    let shared_state = Arc::new(AppState {
        db: Box::new(SqliteDatabase::new("db.sqlite").expect("Failed to create database")),
        crypt_provider: CryptProviders::SimpleCryptProv
    });

    // test code
    shared_state.db.create_table_for_type::<ModuleDB>().unwrap();
    shared_state.db.new_entry::<ModuleDB>(vec![SQLWhereValue::Blob(vec![1,2,3]), SQLWhereValue::Float64(3.4)]).unwrap();
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
