use std::{any::type_name, error::Error, sync::Arc};

use axum::{
    extract::State, http::{HeaderMap, StatusCode}, routing::{delete, get, post}, Json, Router
};
use log::{error, info, warn};
use objects::{CourseDB, CourseRequest, CourseSend, ExamDB, ExamRequest, ExamSend, StudyGoalDB, StudyGoalRequest, StudyGoalSend, ToDoDB, ToDoRequest, ToDoSend, TopicDB, TopicRequest, TopicSend};
use serde::{Deserialize, Serialize};

use crate::{auth_handler::{decrypt_local_token_for, verify_token}, crypt::crypt_provider::CryptProviders, db::{sql_helper::{SQLGenerate, SQLValue}, DBInterface}, db_param_map, AppState};

// allow dead code but only in objects
#[allow(dead_code)]
pub mod objects;

/// This function defines the authentication routes for the application.
pub fn data_router<DB: DBInterface + Send + Sync + 'static>(state: Arc<AppState<DB>>) -> Router {
    // create the db tables
    state.db.create_table_for_type::<CourseDB>().unwrap();
    state.db.create_table_for_type::<TopicDB>().unwrap();
    state.db.create_table_for_type::<StudyGoalDB>().unwrap();
    state.db.create_table_for_type::<ExamDB>().unwrap();
    state.db.create_table_for_type::<ToDoDB>().unwrap();

    // handles returning data
    let get_routes = Router::new()
        .route("/course", get(handle_get::<CourseDB, CourseSend, CourseRequest, DB>))
        .route("/topic", get(handle_get::<TopicDB, TopicSend, TopicRequest, DB>))
        .route("/study_goal", get(handle_get::<StudyGoalDB, StudyGoalSend, StudyGoalRequest, DB>))
        .route("/exam", get(handle_get::<ExamDB, ExamSend, ExamRequest, DB>))
        .route("/todo", get(handle_get::<ToDoDB, ToDoSend, ToDoRequest, DB>));

    // handles creating / editing data
    let new_routes = Router::new()
        .route("/course", post(handle_new::<CourseDB, CourseSend, DB>))
        .route("/topic", post(handle_new::<TopicDB, TopicSend, DB>))
        .route("/study_goal", post(handle_new::<StudyGoalDB, StudyGoalSend, DB>))
        .route("/exam", post(handle_new::<ExamDB, ExamSend, DB>))
        .route("/todo", post(handle_new::<ToDoDB, ToDoSend, DB>));


    // handles deleting data
    let delete_routes = Router::new()
        .route("/course", delete(handle_delete::<CourseDB, DB>))
        .route("/topic", delete(handle_delete::<TopicDB, DB>))
        .route("/study_goal", delete(handle_delete::<StudyGoalDB, DB>))
        .route("/exam", delete(handle_delete::<ExamDB, DB>))
        .route("/todo", delete(handle_delete::<ToDoDB, DB>));

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

// TRAITS that are used for objects
/// structs implementing this trait require an id field and a corresponding SQLGenerate Type, which has a user_id field
/// gets implemented by SendObject derive macro
pub trait Sendable {
    /// gets the id for the send Object
    fn get_id(&self) -> Option<i32>;
    // /// returns a vector of all parameters excluding id
    //fn to_param_vec(&self) -> Vec<(String, SQLValue)>;
}

/// needs to be implemented for every Request type, converts the Option<T> to a map with only some values
/// gets implemented by Selector derive macro
pub trait ToSelect {
    fn to_select_param_vec(&self) -> Vec<(String, SQLValue)>;
}


/// needs to be implemented for every Send datatype, helps converting the send datatype into a parameter map, encrypts values
pub trait ToDB {
    /// should generate a sqlvalue param map, containing every value, besides id and user_id, encrypt as much as possible
    fn to_param_vec(&self, key: &[u8], provider: &CryptProviders) -> Vec<(String, SQLValue)>;
}

/// needs to be implemented for send types
pub trait FromDB<DBT: SQLGenerate> {
    /// should convert a dbt to a Send type, decrypting the crypt values
    fn from_dbt(dbt: &DBT, key: &[u8], provider: &CryptProviders) -> Result<Self, Box<dyn Error>> where Self: Sized;
}

/// handler for get requests, retrieving objects from the db
pub async fn handle_get<DBT: SQLGenerate, ST: FromDB<DBT>, RT: ToSelect, DB: DBInterface + Send + Sync>(
    headers: HeaderMap,
    State(state): State<Arc<AppState<DB>>>,
    Json(request): Json<RT>,
) -> Result<Json<Vec<ST>>, StatusCode> {
    info!("{} read requested!", type_name::<DBT>());

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

    // retrieve db data
    let mut params = db_param_map! { user_id: user_id };
    // only values that have Some(T) are added to the params list
    params.extend(request.to_select_param_vec());

    let entries = state.db.select_entries::<DBT>(params);
    if entries.is_err() {
        error!("Error while querying DB! Tried to get {} information.", type_name::<DBT>());
        return Err(StatusCode::INTERNAL_SERVER_ERROR);
    }

    let entries_send: Result<Vec<ST>, StatusCode> = entries.unwrap().iter().map(|entry| {
        ST::from_dbt(entry, local_token.as_bytes(), &state.crypt_provider).map_err(|_| {
            error!("Failed to convert database type to send type");
            StatusCode::INTERNAL_SERVER_ERROR
        })
    }).collect();
    let entries_send = entries_send?;

    info!("{} read successful, building response!", type_name::<DBT>());
    Ok(Json(entries_send))
}

/// handler for creating new objects
async fn handle_new<DBT: SQLGenerate,ST: Sendable + ToDB, DB: DBInterface + Send + Sync>(
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
        let mut params= db_param_map! { user_id: user_id };
        // extend it with the parameters from the send type (except for user_id)
        params.extend(request.to_param_vec(local_token.as_bytes(), &state.crypt_provider));

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
        let where_params = db_param_map! {
            id: entry_id,
            user_id: user_id,
        };

        // always update every field, retrieved from the request type
        let params = request.to_param_vec(local_token.as_bytes(), &state.crypt_provider);

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
    let result = state.db.delete_entry::<DBT>(db_param_map! { id: request.id, user_id: user_id});

    if result.is_err() {
        // this happens if the sql querry is formatted wrong (which should never happen)
        error!("Failed to delete entry in DB!");
        return Err(StatusCode::INTERNAL_SERVER_ERROR);
    }

    info!("{} deletion successful.", type_name::<DBT>());
    Ok(Json(IDBody {id: request.id}))
}
