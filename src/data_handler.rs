use std::sync::Arc;

use axum::{
    Router,
    routing::{delete, get, post},
};
use serde::{Deserialize, Serialize};

use crate::{AppState, db::DBInterface};

mod course;

/// This function defines the authentication routes for the application.
pub fn data_router<DB: DBInterface + Send + Sync + 'static>(state: Arc<AppState<DB>>) -> Router {
    // handles returning data
    let get_routes = Router::new().route("/course", get(course::handle_get_course));

    // handles creating / editing data
    let new_routes = Router::new().route("/course", post(course::handle_new_course));

    // handles deleting data
    let delete_routes = Router::new().route("/course", delete(course::handle_delete_course));

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
    AuthFailure,
}

// objects

// course: consists of: name (cryptstring)
// topic: consists of: course_id (foreign key), name (cryptstring), (details (cryptstring), if not in study_goal)
// study_goal: consists of: topic_id (foreign key), 
