use std::{any::type_name, sync::Arc};

use axum::{
    extract::State, http::{HeaderMap, StatusCode}, routing::{delete, get, post}, Json, Router
};
use db_derive::SendObject;
use log::{debug, error, info, warn};
use serde::{Deserialize, Serialize};

use crate::{auth_handler::{decrypt_local_token_for, verify_token}, db::{sql_helper::{to_crypt_value, SQLGenerate, SQLValue}, Course, DBInterface}, select_fields, AppState};

mod course;

/// This function defines the authentication routes for the application.
pub fn data_router<DB: DBInterface + Send + Sync + 'static>(state: Arc<AppState<DB>>) -> Router {
    // create the db tables
    state.db.create_table_for_type::<Course>().unwrap();

    // handles returning data
    let get_routes = Router::new().route("/course", get(course::handle_get_course));

    // handles creating / editing data
    let new_routes = Router::new().route("/course", post(handle_new::<Course, CourseSend, DB>));

    // handles deleting data
    let delete_routes = Router::new().route("/course", delete(handle_delete::<Course, DB>));

    Router::new()
        .merge(get_routes)
        .merge(new_routes)
        .merge(delete_routes)
        .with_state(state)
}
// general structs

/// response / request with an id
#[derive(Deserialize, Serialize, Debug)]
struct IDBody {
    id: i32
}

// objects
// FIXME: maybe encrypt dates? booleans?

// course: consists of: name (cryptstring)
// topic: consists of: course_id (foreign key), name (cryptstring), details (cryptstring)
// study_goal: consists of: topic_id (foreign key), deadline (date), 
// exam: consists of: course_id (foreign key), name (cryptstring), date (date)

// todo: consists of: name (cryptstring), deadline (date), details (crypstring), completed (bool)

// generic functions

/// structs implementing this trait require an id field and a corresponding SQLGenerate Type, which has a user_id field
/// gets implemented by SendObject derive macro
pub trait Sendable {
    /// gets the id for the send Object
    fn get_id(&self) -> Option<i32>;
    /// returns a vector of all parameters excluding id
    fn to_param_vec(&self) -> Vec<(String, SQLValue)>;
}

#[derive(Debug, Deserialize, Serialize, SendObject)]
struct CourseSend {
    id: Option<i32>,
    name: String,
}

async fn handle_new<DBT: SQLGenerate,ST: Sendable, DB: DBInterface + Send + Sync>(
    headers: HeaderMap,
    State(state): State<Arc<AppState<DB>>>,
    Json(request): Json<ST>,
) -> Result<Json<IDBody>, StatusCode> {
    info!("{} creation / edit requested!", type_name::<DBT>());

    let auth_header = headers.get("authorization");

    // verify that the token is valid
    let verified_token = verify_token(auth_header, state.clone());
    if verified_token.is_err() {
        warn!("Authentication failure, invalid token!");
        // invalid token, authentication failure
        return Err(StatusCode::UNAUTHORIZED);
    }
    let (user_id, remote_token_id, remote_token) = verified_token.unwrap();

    // decrypt the corresponding local token
    let local_token = decrypt_local_token_for(
        user_id,
        &DBT::get_db_ident(),
        remote_token_id,
        &remote_token,
        state.clone(),
    );
    if local_token.is_err() {
        error!(
            "Failed to decrypt local token with remote token (id: {})",
            remote_token_id
        );
        return Err(StatusCode::INTERNAL_SERVER_ERROR);
    }
    let local_token = local_token.unwrap();

    // id is null => means we want to create
    // not null   => means we want to edit
    if request.get_id().is_none() {
        info!("Authentication successful, creation requested.");

        // insert user id, as this is not included in the send data type
        let mut params= select_fields! { user_id: user_id };
        // extend it with the parameters from the send type (except for user_id)
        // FIXME: not very clean
        params.extend(request.to_param_vec().iter().map(|(k, v)| {
            (k.clone(), to_crypt_value(v, local_token.as_bytes(), state.clone()))
        }).collect::<Vec<(String, SQLValue)>>());

        debug!("{:?}", params);

        let id = state
            .db
            .new_entry::<DBT>(params);
        if id.is_err() {
            error!(
                "Failed to insert new {} into db! (user id: {})",
                type_name::<DBT>(),
                user_id
            );
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
        info!("{} creation successful.", type_name::<DBT>());

        Ok(Json(IDBody { id: id.unwrap() }))
    } else {
        info!("Authentication successful, edit requested.");
        // id is not none
        let entry_id = request.get_id().unwrap();

        // prepare where params (same for every type)
        let where_params = select_fields! {
            id: entry_id,
            user_id: user_id,
        };

        // always update every field, retrieved from the request type
        // FIXME: not clean but encrypts i guess
        let params = request.to_param_vec().iter().map(|(k, v)| {
            (k.clone(), to_crypt_value(v, local_token.as_bytes(), state.clone()))
        }).collect::<Vec<(String, SQLValue)>>();

        let result = state.db.update_entry::<DBT>(params, where_params);
        if result.is_err() {
            error!("Failed to edit {} in DB! {} id: {}", type_name::<DBT>(), type_name::<DBT>(), entry_id);
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }

        info!("{} edit successful.", type_name::<DBT>());
        // respond with the id that we already got from client, but hey we need to send something
        Ok(Json(IDBody { id: entry_id }))
    }
}


/// handles delete request for a type T which has to implement SQLGenerate
/// T also has to have the id and user_id field for this to work, as those two are used to strictly identify an element in the DB
async fn handle_delete<DBT: SQLGenerate, DB: DBInterface + Send + Sync>(
    headers: HeaderMap,
    State(state): State<Arc<AppState<DB>>>,
    Json(request): Json<IDBody>,
) -> Result<Json<IDBody>, StatusCode> {
    info!("{} deletion requested!", type_name::<DBT>());

    let auth_header = headers.get("authorization");

    // verify that the token is valid
    let verified_token = verify_token(auth_header, state.clone());
    if verified_token.is_err() {
        warn!("Authentication failure, invalid token!");
        // invalid token, authentication failure
        return Err(StatusCode::UNAUTHORIZED);
    }
    let (user_id, _, _) = verified_token.unwrap();
    // we do not need a local token, because we do not need to decrypt or encrypt anything

    // all is good, delete the provided entry
    let result = state.db.delete_entry::<DBT>(select_fields! { id: request.id, user_id: user_id});

    if result.is_err() {
        // this happens if the sql querry is formatted wrong (which should never happen)
        error!("Failed to delete entry in DB!");
        return Err(StatusCode::INTERNAL_SERVER_ERROR);
    }

    info!("{} deletion successful.", type_name::<DBT>());
    Ok(Json(IDBody {id: request.id}))
}
