use std::sync::Arc;

use axum::{Json, extract::State, http::HeaderMap};
use log::{error, info, warn};
use serde::{Deserialize, Serialize};

use crate::{
    AppState,
    auth_handler::{decrypt_local_token_for, verify_token},
    crypt::{Cryptable, crypt_types::CryptString},
    db::{DBInterface, DBStructs, Module, sql_helper::SQLWhereValue},
};

use super::EditResponse;

/// struct for sending and receiving the module data type
#[derive(Deserialize, Serialize, Debug)]
pub struct ModuleSend {
    id: i32,
    name: String,
}

pub async fn handle_get_module<DB: DBInterface + Send + Sync>(
    State(state): State<Arc<AppState<DB>>>,
) {
}

pub async fn handle_new_module<DB: DBInterface + Send + Sync>(
    headers: HeaderMap,
    State(state): State<Arc<AppState<DB>>>,
    Json(request): Json<ModuleSend>,
) -> Json<EditResponse> {
    info!("Module creation / edit requested!");

    let auth_header = headers.get("authorization");

    // verify that the token is valid
    let verified_token = verify_token(auth_header, state.clone());
    if verified_token.is_err() {
        warn!("Authentication failure, invalid token!");
        // invalid token, authentication failure
        return Json(EditResponse::AuthFailure);
    }
    let (user_id, remote_token_id, remote_token) = verified_token.unwrap();

    // decrypt the corresponding local token
    let local_token = decrypt_local_token_for(
        user_id,
        &DBStructs::Module,
        remote_token_id,
        &remote_token,
        state.clone(),
    );
    if local_token.is_err() {
        error!(
            "Failed to decrypt local token with remote token (id: {})",
            remote_token_id
        );
        return Json(EditResponse::InternalFailure);
    }
    let local_token = local_token.unwrap();

    // id < 0 => means we want to create
    // id >= 0 means we want to edit
    if request.id < 0 {
        info!("Authentication successful, creation requested.");
        let name =
            CryptString::encrypt(&request.name, local_token.as_bytes(), &state.crypt_provider);

        let id = state
            .db
            .new_entry::<Module>(vec![SQLWhereValue::Blob(name.data_crypt)]);
        if id.is_err() {
            error!(
                "Failed to insert new module into db! (user id: {})",
                user_id
            );
            return Json(EditResponse::InternalFailure);
        }
        info!("Module creation successful.");

        Json(EditResponse::Success(id.unwrap()))
    } else {
        info!("Authentication successful, edit requested.");
        // TODO: edit entry
        Json(EditResponse::Success(0))
    }
}

pub async fn handle_delete_module<DB: DBInterface + Send + Sync>(
    State(state): State<Arc<AppState<DB>>>,
) {
}
