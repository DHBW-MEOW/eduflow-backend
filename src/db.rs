use std::error::Error;

use chrono::NaiveDateTime;
use db_derive::DBObject;
use sql_helper::{SQLGenerate, SQLWhereValue};

use crate::crypt::crypt_types::{CryptI32, CryptString};

pub mod sql_helper;
pub mod sqlite;

/// Database interface trait that defines the methods for database operations.
pub trait DBInterface {
    // AUTH
    
    // user related
    /// create a new user
    fn new_user(&self, username: &str, password_hash: &str) -> Result<(), Box<dyn Error>>;
    /// Get a user by their username.
    fn get_user_by_username(&self, username: &str) -> Result<User, Box<dyn Error>>;

    // token related

    // write tokens
    /// create new password encrypted local token
    fn new_local_token_pwcrypt(&self, user_id: i32, token_crypt: CryptString) -> Result<(), Box<dyn Error>>;
    /// create a new encrypted version of an already existing local token (encrypted by a remote token)
    fn new_local_token_rtcrypt(&self, local_token_id: i32, local_token_crypt: &CryptString, decryptable_by_rt_id: i32, valid_until: &NaiveDateTime) -> Result<(), Box<dyn Error>>;
    /// create new remote token, results in write access, returns remote token id
    fn new_remote_token(&self, rt_hash: &str, user_id: i32) -> Result<i64, Box<dyn Error>>;

    // get tokens
    /// get all local tokens for a user encrypted by password
    fn get_local_tokens_by_user_pwcrypt(&self, user_id: i32) -> Result<Vec<LocalTokenPWCrypt>, Box<dyn Error>>;
    /// get a single local token by id encrypted by password
    fn get_local_token_by_id_pwcrypt(&self, local_token_id: i32) -> Result<LocalTokenPWCrypt, Box<dyn Error>>;
    /// get all local tokens encrypted by a remote token
    //fn get_local_tokens_by_rthash(&self, remote_token_hash: &str) -> Result<Vec<LocalTokenRTCrypt>, Box<dyn Error>>;
    /// get a single local token encrypted by a remote token
    fn get_local_token_by_id_rtcrypt(&self, local_token_id: i32) -> Result<LocalTokenRTCrypt, Box<dyn Error>>;
    /// get remote token by id
    fn get_remote_token(&self, token_id: i32) -> Result<RemoteToken, Box<dyn Error>>;


    // DATA related, using generics and a few macros
    /// creates a new database table for the type T, which has to have the DBObject derive macro
    fn create_table_for_type<T: SQLGenerate>(&self) -> Result<(), Box<dyn Error>>;
    /// enters a new entry into the database table of the type T, a table using create_table_for_type has to be created beforehand.
    fn new_entry<T: SQLGenerate>(&self, params: Vec<SQLWhereValue>) -> Result<i32, Box<dyn Error>>;
}

// structs, which are stored inside of the database

// AUTH
#[derive(Debug)]
pub struct User {
    pub id: i32,
    pub username: String,
    pub password_hash: String,
    pub created_at: NaiveDateTime,
}
/// struct that stores the local tokens encrypted by the users password
#[derive(Debug)]
pub struct LocalTokenPWCrypt {
    pub id: i32,
    pub user_id: i32,
    pub token_crypt: CryptString,
}
/// struct that stores the local tokens encrypted by a remote token
#[derive(Debug)]
pub struct LocalTokenRTCrypt {
    pub id: i32,
    pub local_token_id: i32,
    pub local_token_crypt: CryptString,
    pub decryptable_by_rt_id: i32,
    pub valid_until: NaiveDateTime,
}

#[derive(Debug)]
pub struct RemoteToken {
    pub id: i32,
    pub rt_hash: String,
    pub user_id: i32
} 

// DATA
/// just a testing struct so we can confirm functionallity
#[derive(Debug, DBObject)]
pub struct TestDummy {
    pub id: i32,
    pub name: String,
    pub secret_number: Vec<u8>, // CryptI32,
    pub secret_text: Vec<u8>, // CryptString,
    pub decryptable_by: i32,
}

/// module db entry
#[derive(DBObject)]
pub struct ModuleDB {
    pub id: i32,
    pub name: Vec<u8>, // CryptString,
    pub test_float: f64,
}