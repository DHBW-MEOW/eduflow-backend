use std::sync::Arc;

use axum::{extract::State, http::HeaderMap, routing::{delete, get, post}, Json, Router};
use log::info;
use serde::{Deserialize, Serialize};

use crate::{auth_handler::verify_token, AppState};


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

    // verify that the token is valid
    let verified_token = verify_token(auth_header, state.clone());

    // invalid token, authentication failure
    if verified_token.is_err() {
        return Json(EditResponse::AuthFailure);
    }
    let (user_id, token_id, token) = verified_token.unwrap();
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