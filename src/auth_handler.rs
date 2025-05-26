use std::sync::Arc;

use argon2::{Argon2, PasswordHash, PasswordVerifier};
use axum::{extract::State, routing::get, Json, Router};
use serde::{Deserialize, Serialize};

/// This function defines the authentication routes for the application.
pub fn auth_router(state: Arc<crate::AppState>) -> Router {
    Router::new()
        .route("/login", get(handle_login))
        //.route("/logout", get(|| async { "Logout Endpoint" }))
        //.route("/register", get(|| async { "Register Endpoint" }))
        .with_state(state)
}

/// struct used for login and register body
#[derive(Deserialize, Serialize, Debug)]
struct LoginRequest {
    username: String,
    password: String,
}

/// struct used for logout body
#[derive(Deserialize, Serialize, Debug)]
struct LogoutRequest {
    token: String,
}

/// enum used for login status
#[derive(Deserialize, Serialize, Debug)]
enum LoginStatus {
    Success,
    Failure,
}
/// struct used for token response
#[derive(Deserialize, Serialize, Debug)]
struct TokenResponse {
    login_status: LoginStatus,
    token: Option<String>,
}

/// handler for login requests
async fn handle_login(State(state): State<Arc<crate::AppState>>, Json(request): Json<LoginRequest>) -> Json<TokenResponse> {
    println!("Login request received: {:?}", request);

    let user = state.db.get_user_by_username(&request.username);

    if let Err(_) = user {
        // User has not been found or an error occurred
        return Json(TokenResponse {
            login_status: LoginStatus::Failure,
            token: None,
        });
    }
    let user = user.unwrap();

    // check if the password matches
    let pwd_hash = PasswordHash::new(&user.password_hash).expect("Password Hash corrupted in DB!");
    let result = Argon2::default().verify_password(request.password.as_bytes(), &pwd_hash);

    if let Err(_) = result {
        // Password does not match
        return Json(TokenResponse {
            login_status: LoginStatus::Failure,
            token: None,
        });
    }

    // password matches -> generate token


    Json(TokenResponse { login_status: LoginStatus::Failure, token: None })
    

}
