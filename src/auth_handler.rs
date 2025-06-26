use std::{error::Error, sync::Arc};

use argon2::{
    Argon2,
    password_hash::{PasswordHash, PasswordHasher, PasswordVerifier, Salt, SaltString},
};
use axum::{extract::State, http::{HeaderMap, HeaderValue, StatusCode}, routing::{get, post}, Json, Router};
use chrono::{Days, Utc};
use log::{error, info, warn};
use rand::{TryRngCore, rngs::OsRng};
use serde::{Deserialize, Serialize};
use token_gen::generate_token;

use crate::{crypt::{crypt_types::CryptString, Cryptable}, db::{DBInterface, DBObjIdent}, AppState};

mod token_gen;

const TOKEN_EXPIRE: u64 = 14; // days after which a token expires

/// This function defines the authentication routes for the application.
pub fn auth_router<DB: DBInterface + Send + Sync + 'static>(state: Arc<AppState<DB>>) -> Router {
    Router::new()
        .route("/register", post(handle_register))
        .route("/login", post(handle_login))
        .route("/logout", post(handle_logout)) // logout basically invalidates a existing token
        .route("/verify-token", get(handle_verify)) // verifies that a given token is valid
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

/// struct used for login / register response
#[derive(Deserialize, Serialize, Debug)]
struct LoginResponse {
    token: String,
}

/// handler for logout requests
async fn handle_logout<DB: DBInterface + Send + Sync>(
    headers: HeaderMap,
    State(state): State<Arc<AppState<DB>>>,
) -> Result<(), StatusCode>{
    info!("Logout request received.");

    let auth_header = headers.get("authorization");

    // confirm that the given token is valid, otherwise we do not need to invalidate it, or someone would just be able to invalidate any token with its id
    let (_, token_id, _) = verify_token(auth_header, state.clone()).map_err(|_| StatusCode::UNAUTHORIZED)?;

    invalidate_remote_token(token_id, state).map_err(|_| {
        // well here something has really gone wrong, we could validate the token but are now unable to delete it.
        error!("Failed to invalidate token! token has been verified beforehand, meaning token is still valid!");
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    Ok(())
}

/// handler for verifying the validity of tokens
async fn handle_verify<DB: DBInterface + Send + Sync>(
    headers: HeaderMap,
    State(state): State<Arc<AppState<DB>>>,
) -> Result<(), StatusCode> {
    info!("Token verification requested!");

    let auth_header = headers.get("authorization");

    // confirm that the given token is valid.
    verify_token(auth_header, state.clone()).map_err(|_| StatusCode::UNAUTHORIZED)?;

    Ok(())
}

/// handler for registration requests
async fn handle_register<DB: DBInterface + Send + Sync>(
    State(state): State<Arc<AppState<DB>>>,
    Json(request): Json<LoginRequest>,
) -> Result<Json<LoginResponse>, StatusCode> {
    info!("Register request for new user {}", request.username);
    // generate salt
    let mut salt_bytes = [0u8; Salt::RECOMMENDED_LENGTH];
    let result = OsRng.try_fill_bytes(&mut salt_bytes);
    let salt = SaltString::encode_b64(&salt_bytes);

    // salt generation error
    if result.is_err() || salt.is_err() {
        error!("Failed to generate salt!");
        return Err(StatusCode::INTERNAL_SERVER_ERROR);
    }
    let salt = salt.unwrap();

    let argon2 = Argon2::default();
    let password_hash = argon2.hash_password(request.password.as_bytes(), salt.as_salt());

    // hashing error
    if password_hash.is_err() {
        error!("Failed to hash password!");
        return Err(StatusCode::INTERNAL_SERVER_ERROR);
    }
    let password_hash = password_hash.unwrap();

    let result = state
        .db
        .new_user(&request.username, password_hash.serialize().as_str());

    if result.is_err() {
        info!("User tried to register with already taken username.");
        return Err(StatusCode::CONFLICT);
    }
    let user_id = result.unwrap();

    // all is right -> generate tokens so user can log in immediately

    // generate local tokens for future use, every db ident element gets a local token
    crate::data_handler::objects::get_db_idents().iter().for_each(|variant| {
            let result = add_new_local_token(user_id, &request.password, variant, state.clone());
            if result.is_err() {
                error!("Failed to generate local token for variant {:?}!, user id: {}, registration partially successful!", variant, user_id);
            }
    });

    // generate remote token for immediate use
    let remote_token = create_remote_token(user_id, request.password, state, TOKEN_EXPIRE);

    if remote_token.is_err() {
        // internal decryption error or db error
        error!("Generating remote token failed!");
        return Err(StatusCode::INTERNAL_SERVER_ERROR);
    }
    let remote_token = remote_token.unwrap();

    info!("Registered new user {}", request.username);

    // build response
    Ok(Json(LoginResponse {
        token: remote_token,
    }))
}

/// handler for login requests
async fn handle_login<DB: DBInterface + Send + Sync>(
    State(state): State<Arc<AppState<DB>>>,
    Json(request): Json<LoginRequest>,
) -> Result<Json<LoginResponse>, StatusCode> {
    info!("Login request from user {}", request.username);

    let user = state.db.get_user_by_username(&request.username);

    if user.is_err() {
        // User has not been found or an error occurred
        // prevent timing attacks and hash the password anyways
        // dummy salt, has no meaning
        let mut dummy_salt_bytes = [0u8; Salt::RECOMMENDED_LENGTH];
        OsRng.try_fill_bytes(&mut dummy_salt_bytes).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
        let dummy_salt = SaltString::encode_b64(&dummy_salt_bytes).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
        let _ = Argon2::default().hash_password(request.password.as_bytes(), dummy_salt.as_salt());
        
        warn!("User tried to log in with non existent user {}.\nPotential brute-force attack, watch out for too many of these warnings.", request.username);
        return Err(StatusCode::UNAUTHORIZED);
    }
    let user = user.unwrap();

    // check if the password matches
    let pwd_hash = PasswordHash::new(&user.password_hash).expect("Password Hash corrupted in DB!");
    let result = Argon2::default().verify_password(request.password.as_bytes(), &pwd_hash);

    if result.is_err() {
        warn!("User {} entered wrong password!", request.username);
        return Err(StatusCode::UNAUTHORIZED);
    }

    // password matches -> generate token
    let remote_token = create_remote_token(user.id, request.password, state, TOKEN_EXPIRE);

    if remote_token.is_err() {
        // internal decryption error or db error
        error!("Generating remote token failed!");
        return Err(StatusCode::INTERNAL_SERVER_ERROR);
    }
    let remote_token = remote_token.unwrap();

    info!("Login successful, returning new remote token to Client!");

    // build response
    Ok(Json(LoginResponse {
        token: remote_token,
    }))
}

/// creates a new remote token for the given user
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
    let remote_token_id = state.db.new_remote_token(&token_hashed, user_id, &valid_until)?;

    
    // re-encrypt every local-token the user possesses, this can also be limited to only some local-tokens to restrict permissions
    state.db.get_local_tokens_by_user_pwcrypt(user_id)?.iter().try_for_each(|lt| {
        let local_token = lt.token_crypt.decrypt(password.as_bytes(), &state.crypt_provider)?;

        let newcrypt_token = CryptString::encrypt(&local_token, remote_token.as_bytes(), &state.crypt_provider);
        state.db.new_local_token_rtcrypt(lt.id, &newcrypt_token, remote_token_id.try_into().expect("Remote token ID is too big!"))?;

        Ok::<(), Box<dyn Error>>(())
    })?;

    // prefix the token with its token id
    let remote_token = remote_token_id.to_string() + "_" + &remote_token;

    Ok(remote_token)
}

fn invalidate_remote_token<DB: DBInterface + Send + Sync>(remote_token_id: i32, state: Arc<AppState<DB>>) -> Result<(), Box<dyn Error>> {
    state.db.del_local_token_rtcrypt_by_rt(remote_token_id)?;
    state.db.del_remote_token(remote_token_id)?;

    Ok(())
}

/// parses and extracts the token and token id from authentication header
fn split_auth_header(auth_header: &str) -> Result<(i32, String), Box<dyn Error>> {
    // check for Bearer token
    let token = auth_header.strip_prefix("Bearer ").ok_or("Invalid Token")?;

    // split the user id 
    let split: Vec<&str> = token.split_terminator("_").collect();

    let token_id = split.first().ok_or("Invalid Token")?;
    let token = split.get(1).ok_or("Invalid Token")?;

    // convert user id to i32
    Ok((token_id.parse()?, token.to_string()))

}

/// verifies if the token is valid
/// returns user_id, token_id and the token itself on success
/// will return err if token is invalid or expired
/// will delete the token entry if expired
pub fn verify_token<DB: DBInterface + Send + Sync>(auth_header: Option<&HeaderValue>, state: Arc<AppState<DB>>) -> Result<(i32, i32, String), Box<dyn Error>> {
    // auth header validation
    let auth_header = auth_header.ok_or("Invalid Token")?.to_str()?;

    // parse the auth header
    let (token_id, token) = split_auth_header(auth_header)?;

    // get the stored token hash
    let token_db = state.db.get_remote_token(token_id)?;

    // Token is no longer valid:
    if token_db.valid_until <= Utc::now().naive_utc() {
        info!("Remote token expired, deleting corresponding entries!");

        // invalidate remote token
        invalidate_remote_token(token_id, state)?;

        return Err("Token expired".into());
    }

    // confirm that the token matches
    let db_token_hash = PasswordHash::new(&token_db.rt_hash).expect("Token Hash corrupted in DB!");
    let result = Argon2::default().verify_password(token.as_bytes(), &db_token_hash);

    match result {
        Ok(_) => Ok((token_db.user_id, token_id, token)),
        Err(_) => Err("Invalid Token".into()),
    }

}
/// takes a remote token, the according user id and used for attribute and decrypts the corresponding local token and returns it
pub fn decrypt_local_token_for<DB: DBInterface + Send + Sync>(user_id: i32, used_for: &DBObjIdent, remote_token_id: i32, remote_token: &str, state: Arc<AppState<DB>>) -> Result<String, Box<dyn Error>>{
    // get the necessary local token and decrypt it
    let local_token_pwcrypt = state.db.get_local_token_by_used_for_pwcrypt(user_id, used_for)?;
    // get the rt encrypted version of it:
    let local_token_rtcrypt = state.db.get_local_token_by_id_rtcrypt(local_token_pwcrypt.id, remote_token_id)?;
    
    // decrypt the local token
    let local_token = local_token_rtcrypt.local_token_crypt.decrypt(remote_token.as_bytes(), &state.crypt_provider)?;
    
    Ok(local_token)
}

/// generates and adds a password encrypted local token to the Database
pub fn add_new_local_token<DB: DBInterface + Send + Sync>(user_id: i32, password: &str, used_for: &DBObjIdent, state: Arc<AppState<DB>>) -> Result<(), Box<dyn Error>>{
    let local_token = generate_token();
    let local_token_crypt = CryptString::encrypt(&local_token, password.as_bytes(), &state.crypt_provider);

    state.db.new_local_token_pwcrypt(user_id, &local_token_crypt, used_for)?;
    Ok(())
}