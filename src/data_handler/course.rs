use std::sync::Arc;

use axum::{extract::State, http::{HeaderMap, StatusCode}, Json};
use log::{error, info, warn};
use serde::{Deserialize, Serialize};

use crate::{
    auth_handler::{decrypt_local_token_for, verify_token}, crypt::{crypt_types::CryptString, Cryptable}, db::{sql_helper::SQLWhereValue, Course, DBInterface, DBStructs}, select_fields, AppState
};

use super::IDResponse;

/// struct for sending and receiving the course data type
#[derive(Deserialize, Serialize, Debug)]
pub struct CourseSend {
    id: Option<i32>,
    name: String,
}
/// struct for requesting a course (or multiple)
#[derive(Deserialize, Serialize, Debug)]
pub struct CourseRequest {
    id: Option<i32>,
}

pub async fn handle_get_course<DB: DBInterface + Send + Sync>(
    headers: HeaderMap,
    State(state): State<Arc<AppState<DB>>>,
    Json(request): Json<CourseRequest>,
) -> Result<Json<Vec<CourseSend>>, StatusCode> {
    info!("Course read requested!");

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
        &DBStructs::Course,
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
    // check if id is null => get all courses for the current user otherwise filter id
    let params = if request.id.is_none() {
        select_fields! {
            user_id: user_id,
        }
    } else {
        select_fields! {
            user_id: user_id,
            id: request.id.unwrap(),
        }
    };

    let entries = state.db.select_entries::<Course>(params);
    if entries.is_err() {
        error!("Error while querying DB! Tried to get Course information.");
        return Err(StatusCode::INTERNAL_SERVER_ERROR);
    }

    let entries_send: Result<Vec<CourseSend>, StatusCode> = entries.unwrap().iter().map(|course| {
        let name = course.name.decrypt(local_token.as_bytes(), &state.crypt_provider);
        if name.is_err() {
            error!("Error while decrypting Course!");
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
        Ok(CourseSend {
            id: Some(course.id),
            name: name.unwrap()
        })
    }).collect();
    let entries_send = entries_send?;

    info!("Course read successful, building response!");
    Ok(Json(entries_send))
}

pub async fn handle_new_course<DB: DBInterface + Send + Sync>(
    headers: HeaderMap,
    State(state): State<Arc<AppState<DB>>>,
    Json(request): Json<CourseSend>,
) -> Result<Json<IDResponse>, StatusCode> {
    info!("Course creation / edit requested!");

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
        &DBStructs::Course,
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
    if request.id.is_none() {
        info!("Authentication successful, creation requested.");
        let name =
            CryptString::encrypt(&request.name, local_token.as_bytes(), &state.crypt_provider);

        let id = state
            .db
            .new_entry::<Course>(vec![SQLWhereValue::Blob(name.data_crypt), SQLWhereValue::Int32(user_id)]);
        if id.is_err() {
            error!(
                "Failed to insert new course into db! (user id: {})",
                user_id
            );
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
        info!("Course creation successful.");

        Ok(Json(IDResponse { id: id.unwrap() }))
    } else {
        info!("Authentication successful, edit requested.");
        // id is not none
        let entry_id = request.id.unwrap();

        // prepare where params
        let where_params = select_fields! {
            id: entry_id,
            user_id: user_id,
        };

        // always update every field, even though we do not really have to
        let name = CryptString::encrypt(&request.name, local_token.as_bytes(), &state.crypt_provider);
        let params = select_fields! {
            name: name.data_crypt,
        };

        let result = state.db.update_entry::<Course>(params, where_params);
        if result.is_err() {
            error!("Failed to edit course in DB! course id: {}", entry_id);
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }

        info!("Course edit successful.");
        // respond with the id that we already got from client, but hey we need to send something
        Ok(Json(IDResponse { id: entry_id }))
    }
}

pub async fn handle_delete_course<DB: DBInterface + Send + Sync>(
    State(state): State<Arc<AppState<DB>>>,
) {
}
