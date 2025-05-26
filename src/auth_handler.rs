use std::sync::Arc;

use argon2::{
    Argon2,
    password_hash::{PasswordHash, PasswordHasher, PasswordVerifier, Salt, SaltString},
};
use axum::{Json, Router, extract::State, routing::get};
use rand::{TryRngCore, rngs::OsRng};
use serde::{Deserialize, Serialize};

/// This function defines the authentication routes for the application.
pub fn auth_router(state: Arc<crate::AppState>) -> Router {
    Router::new()
        .route("/register", get(handle_register))
        .route("/login", get(handle_login))
        //.route("/logout", get(|| async { "Logout Endpoint" })) // logout basically invalidates a existing token
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
/// struct used for login response
#[derive(Deserialize, Serialize, Debug)]
struct LoginResponse {
    login_status: LoginStatus,
    token: Option<String>,
}

/// enum used for register status
#[derive(Deserialize, Serialize, Debug)]
enum RegisterStatus {
    Success,
    InternalFailure,
    UsernameTakenFailure,
}
/// struct used for register response
#[derive(Deserialize, Serialize, Debug)]
struct RegisterResponse {
    register_status: RegisterStatus,
    token: Option<String>,
}

/// handler for registration requests
async fn handle_register(
    State(state): State<Arc<crate::AppState>>,
    Json(request): Json<LoginRequest>,
) -> Json<RegisterResponse> {
    // generate salt
    let mut salt_bytes = [0u8; Salt::RECOMMENDED_LENGTH];
    let result = OsRng.try_fill_bytes(&mut salt_bytes);
    let salt = SaltString::encode_b64(&salt_bytes);

    // salt generation error
    if result.is_err() || salt.is_err() {
        return Json(RegisterResponse {
            register_status: RegisterStatus::InternalFailure,
            token: None,
        });
    }
    let salt = salt.unwrap();

    let argon2 = Argon2::default();
    let password_hash = argon2.hash_password(request.password.as_bytes(), salt.as_salt());

    // hashing error
    if password_hash.is_err() {
        return Json(RegisterResponse {
            register_status: RegisterStatus::InternalFailure,
            token: None,
        });
    }
    let password_hash = password_hash.unwrap();

    let result = state
        .db
        .new_user(&request.username, password_hash.serialize().as_str());

    // if this fails, the username is already taken
    if result.is_err() {
        return Json(RegisterResponse {
            register_status: RegisterStatus::UsernameTakenFailure,
            token: None,
        });
    }

    // all is right -> generate token so user can log in immedieately
    // TODO: Generate token

    Json(RegisterResponse {
        register_status: RegisterStatus::Success,
        token: None,
    })
}

/// handler for login requests
async fn handle_login(
    State(state): State<Arc<crate::AppState>>,
    Json(request): Json<LoginRequest>,
) -> Json<LoginResponse> {
    println!("Login request received: {:?}", request);

    let user = state.db.get_user_by_username(&request.username);

    if user.is_err() {
        // User has not been found or an error occurred
        return Json(LoginResponse {
            login_status: LoginStatus::Failure,
            token: None,
        });
    }
    let user = user.unwrap();

    // check if the password matches
    let pwd_hash = PasswordHash::new(&user.password_hash).expect("Password Hash corrupted in DB!");
    let result = Argon2::default().verify_password(request.password.as_bytes(), &pwd_hash);

    if result.is_err() {
        // Password does not match
        return Json(LoginResponse {
            login_status: LoginStatus::Failure,
            token: None,
        });
    }

    // password matches -> generate token
    // TODO: generate token

    Json(LoginResponse {
        login_status: LoginStatus::Success,
        token: None,
    })
}
