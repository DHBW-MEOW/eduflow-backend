use std::sync::Arc;

use axum::{extract::State, http::HeaderMap, routing::{delete, get, post}, Json, Router};
use log::{error, info};
use serde::{Deserialize, Serialize};

use crate::{auth_handler::{decrypt_local_token_for, verify_token}, crypt::{crypt_types::CryptString, Cryptable}, db::{sql_helper::SQLWhereValue, DBInterface, DBStructs, Module}, AppState};


/// This function defines the authentication routes for the application.
pub fn data_router<DB: DBInterface + Send + Sync + 'static>(state: Arc<AppState<DB>>) -> Router {

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
    Success(i32),
    IDNotFound,
    InternalFailure,
    AuthFailure
}

/// struct for sending and receiving the module data type
#[derive(Deserialize, Serialize, Debug)]
struct ModuleSend {
    id: i32,
    name: String,
}

async fn handle_get_module<DB: DBInterface + Send + Sync>(State(state): State<Arc<AppState<DB>>>) {

}

async fn handle_new_module<DB: DBInterface + Send + Sync>(headers: HeaderMap, State(state): State<Arc<AppState<DB>>>, Json(request): Json<ModuleSend>) -> Json<EditResponse>{
    info!("{:?}", headers);

    let auth_header = headers.get("authorization");

    // verify that the token is valid
    let verified_token = verify_token(auth_header, state.clone());
    if verified_token.is_err() {
        // invalid token, authentication failure
        return Json(EditResponse::AuthFailure);
    }
    let (user_id, remote_token_id, remote_token) = verified_token.unwrap();
    
    // decrypt the corresponding local token
    let local_token = decrypt_local_token_for(user_id, &DBStructs::Module, remote_token_id, &remote_token, state.clone());
    if local_token.is_err() {
        error!("Failed to decrypt local token with remote token (id: {})", remote_token_id);
        return Json(EditResponse::InternalFailure);
    }
    let local_token = local_token.unwrap();

    // id < 0 => means we want to create
    // id >= 0 means we want to edit
    if request.id < 0 {
        let name = CryptString::encrypt(&request.name, local_token.as_bytes(), &state.crypt_provider);

        let id = state.db.new_entry::<Module>(vec![SQLWhereValue::Blob(name.data_crypt)]);
        if id.is_err() {
            error!("Failed to insert new module into db! (user id: {})", user_id);
            return Json(EditResponse::InternalFailure);
        }
        Json(EditResponse::Success(id.unwrap()))
    } else {
        // TODO: edit entry
        Json(EditResponse::Success(0))
    }

}

async fn handle_delete_module<DB: DBInterface + Send + Sync>(State(state): State<Arc<AppState<DB>>>) {

}


// objects

// module : consists of name and 