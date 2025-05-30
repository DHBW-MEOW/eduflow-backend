use std::sync::Arc;

use axum::{extract::State, http::HeaderMap, routing::{delete, get, post}, Json, Router};
use log::{error, info};
use serde::{Deserialize, Serialize};

use crate::{auth_handler::verify_token, crypt::{crypt_types::CryptString, Cryptable}, db::{sql_helper::SQLWhereValue, DBInterface, DBStructs, Module}, AppState};


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
    
    // get the neccessary local token and decrypt it
    let local_token_pwcrypt = state.db.get_local_token_by_used_for_pwcrypt(user_id, &DBStructs::Module);
    if local_token_pwcrypt.is_err() {
        // there are multiple local tokens for the same purpose and user or something else has gone wrong
        error!("Could not retrieve local token for user id: {}, used for Module! Local tokens corrupted!", user_id);
        return Json(EditResponse::InternalFailure);
    }
    let local_token_pwcrypt = local_token_pwcrypt.unwrap();

    // get the rt encrypted version of it:
    let local_token_rtcrypt = state.db.get_local_token_by_id_rtcrypt(local_token_pwcrypt.id, remote_token_id);
    if local_token_rtcrypt.is_err() {
        // the version encrypted by the remote token is missing
        error!("Could not retrieve local token (id: {}) encrypted by remote token (id: {}), encrypted version missing?", local_token_pwcrypt.id, remote_token_id);
        return Json(EditResponse::InternalFailure);
    }
    let local_token_rt_crypt = local_token_rtcrypt.unwrap();
    
    // decrypt the local token
    let local_token = local_token_rt_crypt.local_token_crypt.decrypt(remote_token.as_bytes(), &state.crypt_provider);
    if local_token.is_err() {
        // decryption failure
        error!("Failed to decrypt local token (id:{})", local_token_rt_crypt.local_token_id);
        return Json(EditResponse::InternalFailure)
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