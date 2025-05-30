use std::{error::Error, sync::Arc};

use argon2::{
    Argon2,
    password_hash::{PasswordHash, PasswordHasher, PasswordVerifier, Salt, SaltString},
};
use axum::{extract::State, http::HeaderValue, routing::get, Json, Router};
use chrono::{Days, Utc};
use log::{debug, error, info, warn};
use rand::{TryRngCore, rngs::OsRng};
use serde::{Deserialize, Serialize};
use token_gen::generate_token;

use crate::{crypt::{crypt_types::CryptString, Cryptable}, db::{DBInterface, DBStructs}, AppState};

mod token_gen;

/// This function defines the authentication routes for the application.
pub fn auth_router<DB: DBInterface + Send + Sync + 'static>(state: Arc<AppState<DB>>) -> Router {
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
    InternalFailure,
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
async fn handle_register<DB: DBInterface + Send + Sync>(
    State(state): State<Arc<AppState<DB>>>,
    Json(request): Json<LoginRequest>,
) -> Json<RegisterResponse> {
    info!("Register request for new user {}", request.username);
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
    let user_id = result.unwrap();

    // all is right -> generate tokens so user can log in immedieately

    // generate local tokens for future use, every DBStructs element gets a local token
    DBStructs::get_iter().for_each(|variant| {
            let result = add_new_local_token(user_id, &request.password, variant, state.clone());
            if result.is_err() {
                error!("Failed to generate local token!, user id: {}, registration partially successfull!", user_id);
            }
    });

    // generate remote token for immediate use
    let remote_token = create_remote_token(user_id, request.password, state, TOKEN_EXPIRE);

    if remote_token.is_err() {
        // internal decryption error or db error
        error!("Generating remote token failed!");
        return Json(RegisterResponse {
            register_status: RegisterStatus::InternalFailure,
            token: None,
        });
    }
    let remote_token = remote_token.unwrap();

    info!("Registered new user {}", request.username);

    // build response
    Json(RegisterResponse {
        register_status: RegisterStatus::Success,
        token: Some(remote_token),
    })
}

const TOKEN_EXPIRE: u64 = 14; // days after which a token expires
/// handler for login requests
async fn handle_login<DB: DBInterface + Send + Sync>(
    State(state): State<Arc<AppState<DB>>>,
    Json(request): Json<LoginRequest>,
) -> Json<LoginResponse> {
    info!("Login request from user {}", request.username);

    let user = state.db.get_user_by_username(&request.username);

    if user.is_err() {
        // User has not been found or an error occurred
        // FIXME: prevent username bruteforce (artificial delay)
        return Json(LoginResponse {
            login_status: LoginStatus::Failure,
            token: None,
        });
    }
    let user = user.unwrap();

    debug!("User found! {:?}", user); // FIXME: maybe not print the password hash

    // check if the password matches
    let pwd_hash = PasswordHash::new(&user.password_hash).expect("Password Hash corrupted in DB!");
    let result = Argon2::default().verify_password(request.password.as_bytes(), &pwd_hash);

    if result.is_err() {
        warn!("User {} entered wrong password!", request.username);

        // Password does not match
        return Json(LoginResponse {
            login_status: LoginStatus::Failure,
            token: None,
        });
    }

    // password matches -> generate token
    let remote_token = create_remote_token(user.id, request.password, state, TOKEN_EXPIRE);

    if remote_token.is_err() {
        // internal decryption error or db error
        error!("Generating remote token failed!");
        return Json(LoginResponse {
            login_status: LoginStatus::InternalFailure,
            token: None,
        });
    }
    let remote_token = remote_token.unwrap();

    // build response
    Json(LoginResponse {
        login_status: LoginStatus::Success,
        token: Some(remote_token),
    })
}


fn create_remote_token<DB: DBInterface + Send + Sync>(user_id: i32, password: String, state: Arc<AppState<DB>>, valid_days: u64) -> Result<String, Box<dyn Error>> {
    let remote_token = generate_token();

    let valid_until = Utc::now().naive_utc() + Days::new(valid_days);

    // hash the token
    // generate salt
    let mut salt_bytes = [0u8; Salt::RECOMMENDED_LENGTH];
    OsRng.try_fill_bytes(&mut salt_bytes)?;
    let salt = SaltString::encode_b64(&salt_bytes);
    // salting problem occurred
    if salt.is_err() {
        return Err("salting failed".into());
    }
    let salt = salt.unwrap();

    let argon2 = Argon2::default();
    let token_hashed = argon2.hash_password(remote_token.as_bytes(), salt.as_salt());
    // hashing error
    if token_hashed.is_err() {
        return Err("hashing failed".into());
    }
    let token_hashed = token_hashed.unwrap().to_string();

    // insert hashed token into db
    let remote_token_id = state.db.new_remote_token(&token_hashed, user_id)?;

    
    // re-encrypt every local-token the user posseses, this can also be limited to only some local-tokens to restrict permissions
    state.db.get_local_tokens_by_user_pwcrypt(user_id)?.iter().try_for_each(|lt| {
        let local_token = lt.token_crypt.decrypt(password.as_bytes(), &state.crypt_provider)?;

        let newcrypt_token = CryptString::encrypt(&local_token, remote_token.as_bytes(), &state.crypt_provider);
        state.db.new_local_token_rtcrypt(lt.id, &newcrypt_token, remote_token_id.try_into().expect("Remote token ID is too big!"),  &valid_until)?;

        Ok::<(), Box<dyn Error>>(())
    })?;

    // prefix the token with its token id
    let remote_token = remote_token_id.to_string() + "_" + &remote_token;

    Ok(remote_token)
}

/// parses and extracts the token and token id from authentication header
fn split_auth_header(auth_header: &str) -> Result<(i32, String), Box<dyn Error>> {
    // check for Bearer token
    let token = auth_header.strip_prefix("Bearer ").ok_or("Invalid Token")?;

    // split the user id 
    let split: Vec<&str> = token.split_terminator("_").collect();

    let token_id = split.get(0).ok_or("Invalid Token")?;
    let token = split.get(1).ok_or("Invalid Token")?;

    // convert user id to i32
    Ok((token_id.parse()?, token.to_string()))

}

/// verifies if the token is valid
/// returns user_id, token_id and the token itself on success
/// will return err if token is invalid]
pub fn verify_token<DB: DBInterface + Send + Sync>(auth_header: Option<&HeaderValue>, state: Arc<AppState<DB>>) -> Result<(i32, i32, String), Box<dyn Error>> {
    // TODO check token expiry date
    // auth header validation
    let auth_header = auth_header.ok_or("Invalid Token")?.to_str()?;

    // parse the auth header
    let (token_id, token) = split_auth_header(auth_header)?;

    // get the stored token hash
    let token_db = state.db.get_remote_token(token_id)?;

    // confirm that the token matches
    let db_token_hash = PasswordHash::new(&token_db.rt_hash).expect("Token Hash corrupted in DB!");
    let result = Argon2::default().verify_password(token.as_bytes(), &db_token_hash);

    match result {
        Ok(_) => Ok((token_db.user_id, token_id, token)),
        Err(_) => Err("Invalid Token".into()),
    }

}
/// takes a remote token, the according user id and used for attribute and decrypts the corresponding local token and returns it
pub fn decrypt_local_token_for<DB: DBInterface + Send + Sync>(user_id: i32, used_for: &DBStructs, remote_token_id: i32, remote_token: &str, state: Arc<AppState<DB>>) -> Result<String, Box<dyn Error>>{
    // get the neccessary local token and decrypt it
    let local_token_pwcrypt = state.db.get_local_token_by_used_for_pwcrypt(user_id, used_for)?;
    // get the rt encrypted version of it:
    let local_token_rtcrypt = state.db.get_local_token_by_id_rtcrypt(local_token_pwcrypt.id, remote_token_id)?;
    
    // decrypt the local token
    let local_token = local_token_rtcrypt.local_token_crypt.decrypt(remote_token.as_bytes(), &state.crypt_provider)?;
    
    Ok(local_token)
}

/// generates and adds a password encrypted local token to the Database
pub fn add_new_local_token<DB: DBInterface + Send + Sync>(user_id: i32, password: &str, used_for: &DBStructs, state: Arc<AppState<DB>>) -> Result<(), Box<dyn Error>>{
    let local_token = generate_token();
    let local_token_crypt = CryptString::encrypt(&local_token, password.as_bytes(), &state.crypt_provider);

    state.db.new_local_token_pwcrypt(user_id, &local_token_crypt, used_for)?;
    Ok(())
}