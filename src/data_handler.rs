use std::sync::Arc;

use axum::{
    Router,
    routing::{delete, get, post},
};
use serde::{Deserialize, Serialize};

use crate::{AppState, db::DBInterface};

mod module;

/// This function defines the authentication routes for the application.
pub fn data_router<DB: DBInterface + Send + Sync + 'static>(state: Arc<AppState<DB>>) -> Router {
    // handles returning data
    let get_routes = Router::new().route("/module", get(module::handle_get_module));

    // handles creating / editing data
    let new_routes = Router::new().route("/module", post(module::handle_new_module));

    // handles deleting data
    let delete_routes = Router::new().route("/module", delete(module::handle_delete_module));

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

// module : consists of name
