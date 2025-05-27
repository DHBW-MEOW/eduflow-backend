use std::sync::Arc;

use axum::{extract::State, http::HeaderMap, routing::{delete, get, post}, Json, Router};
use log::info;
use serde::{Deserialize, Serialize};

use crate::{auth_handler::{split_auth_header, verify_user}, AppState};


/// This function defines the authentication routes for the application.
pub fn data_router(state: Arc<crate::AppState>) -> Router {

    // handles returning data
    let get_routes = Router::new()
        .route("/module", get(handle_get_module));

    // handles creating / editing data
    let new_routes = Router::new()
        .route("/module", post(handle_new_module));

    // handles deleting data
    let delete_routes = Router::new()
        .route("/module", delete(handle_delete_module));

    Router::new()
        .merge(get_routes)
        .merge(new_routes)
        .merge(delete_routes)
        .with_state(state)
}
// general structs
#[derive(Deserialize, Serialize, Debug)]
enum EditResponse {
    Success,
    IDNotFound,
    AuthFailure
}

/// struct for sending and receiving the module data type
#[derive(Deserialize, Serialize, Debug)]
struct ModuleSend {
    id: i32,
    name: String,
}

async fn handle_get_module(State(state): State<Arc<AppState>>) {

}

async fn handle_new_module(headers: HeaderMap, State(state): State<Arc<AppState>>, Json(request): Json<ModuleSend>) -> Json<EditResponse>{
    info!("{:?}", headers);

    let auth_header = headers.get("authorization");

    // auth header missing
    if auth_header.is_none() {
        return Json(EditResponse::AuthFailure);
    }
    let auth_header = auth_header.unwrap().to_str();

    // auth header not consisting of characters
    if auth_header.is_err() {
        return Json(EditResponse::AuthFailure);
    }
    let auth_header = auth_header.unwrap();

    let split_token = split_auth_header(auth_header);

    // invalid token format
    if split_token.is_err() {
        return Json(EditResponse::AuthFailure);
    }
    let split_token = split_token.unwrap();
    // 




    // id < 0 => means we want to create
    // id >= 0 means we want to edit

    if request.id < 0 {

    } else {

    }

    Json(EditResponse::Success)
}

async fn handle_delete_module(State(state): State<Arc<AppState>>) {

}


// objects

// module : consists of name and 