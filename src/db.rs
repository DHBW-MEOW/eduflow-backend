use std::{error::Error, slice::Iter};

use chrono::{NaiveDate, NaiveDateTime};
use db_derive::DBObject;
use sql_helper::{SQLGenerate, SQLValue};

use crate::crypt::crypt_types::CryptString;

pub mod sql_helper;
pub mod sqlite;

/// Database interface trait that defines the methods for database operations.
pub trait DBInterface {
    // AUTH
    
    // user related
    /// create a new user, returns the user id
    fn new_user(&self, username: &str, password_hash: &str) -> Result<i32, Box<dyn Error>>;
    /// Get a user by their username.
    fn get_user_by_username(&self, username: &str) -> Result<User, Box<dyn Error>>;

    // token related

    // write tokens
    /// create new password encrypted local token
    fn new_local_token_pwcrypt(&self, user_id: i32, token_crypt: &CryptString, used_for: &DBObjIdent) -> Result<(), Box<dyn Error>>;
    /// create a new encrypted version of an already existing local token (encrypted by a remote token)
    fn new_local_token_rtcrypt(&self, local_token_id: i32, local_token_crypt: &CryptString, decryptable_by_rt_id: i32, valid_until: &NaiveDateTime) -> Result<(), Box<dyn Error>>;
    /// create new remote token, results in write access, returns remote token id
    fn new_remote_token(&self, rt_hash: &str, user_id: i32) -> Result<i64, Box<dyn Error>>;

    // get tokens
    /// get all local tokens for a user encrypted by password
    fn get_local_tokens_by_user_pwcrypt(&self, user_id: i32) -> Result<Vec<LocalTokenPWCrypt>, Box<dyn Error>>;
    /// get a single local token by id encrypted by password
    fn get_local_token_by_used_for_pwcrypt(&self, user_id: i32, used_for: &DBObjIdent) -> Result<LocalTokenPWCrypt, Box<dyn Error>>;
    /// get all local tokens encrypted by a remote token
    //fn get_local_tokens_by_rthash(&self, remote_token_hash: &str) -> Result<Vec<LocalTokenRTCrypt>, Box<dyn Error>>;
    /// get a single local token encrypted by a remote token
    fn get_local_token_by_id_rtcrypt(&self, local_token_id: i32, remote_token_id: i32) -> Result<LocalTokenRTCrypt, Box<dyn Error>>;
    /// get remote token by id
    fn get_remote_token(&self, token_id: i32) -> Result<RemoteToken, Box<dyn Error>>;


    // DATA related, using generics and a few macros
    /// creates a new database table for the type T, which has to have the DBObject derive macro
    fn create_table_for_type<T: SQLGenerate>(&self) -> Result<(), Box<dyn Error>>;
    /// enters a new entry into the database table of the type T, a table using create_table_for_type has to be created beforehand.
    fn new_entry<T: SQLGenerate>(&self, params: Vec<(String, SQLValue)>) -> Result<i32, Box<dyn Error>>;
    /// selects entries with where statement depending on which params are passed
    fn select_entries<T: SQLGenerate>(&self, params: Vec<(String, SQLValue)>) -> Result<Vec<T>, Box<dyn Error>>;
    /// udpates a single row, params are the changed parameters, where_params is the WHERE statement which selects what rows to update
    fn update_entry<T: SQLGenerate>(&self, params: Vec<(String, SQLValue)>, where_params: Vec<(String, SQLValue)>) -> Result<(), Box<dyn Error>>;
    /// deletes one or more entries, params determines the where clause which selects what entries to delete
    fn delete_entry<T: SQLGenerate>(&self, params: Vec<(String, SQLValue)>) -> Result<(), Box<dyn Error>>;
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
    pub used_for: DBObjIdent,
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
pub struct Test {
    pub test: &'static str
}

// DATA
pub fn get_db_idents() -> [DBObjIdent; 1] {
    [Course::get_db_ident()]
}
#[derive(Debug)]
pub struct DBObjIdent {
    pub db_identifier: String,
}
/// enum of all data entries the user can access
/// used for generating local tokens (one local token per user per db struct)
#[derive(Debug)]
pub enum DBStructs {
    Course,
}

impl DBStructs {
    /// get all db structs as iterator
    pub fn get_iter() -> Iter<'static, DBStructs> {
        // NOTE: this array has to be manually extended
        // every enum element needs to be in here
        static DB_STRUCTS: [DBStructs; 1] = [DBStructs::Course];
        DB_STRUCTS.iter()
    }
}

impl From<String> for DBStructs {
    fn from(value: String) -> Self {
        match value.as_str() {
            "Course" => DBStructs::Course,
            _ => panic!()
        }
    }
}

impl ToString for DBStructs {
    fn to_string(&self) -> String {
        match self {
            DBStructs::Course => "Course".to_string(),
        }
    }
}

// db entries
#[derive(Debug, DBObject)]
pub struct Course {
    pub id: i32,
    pub user_id: i32,

    pub name: CryptString,
}
#[derive(Debug, DBObject)]
pub struct Topic {
    pub id: i32,
    pub user_id: i32,

    pub course_id: i32,
    pub name: CryptString,
    pub details: CryptString,
}
#[derive(Debug, DBObject)]
pub struct StudyGoal {
    pub id: i32,
    pub user_id: i32,

    pub topic_id: i32,
    pub deadline: NaiveDate,
}
#[derive(Debug, DBObject)]
pub struct Exam {
    pub id: i32,
    pub user_id: i32,

    pub course_id: i32,
    pub name: CryptString,
    pub date: NaiveDate,
}
#[derive(Debug, DBObject)]
pub struct ToDo {
    pub id: i32,
    pub user_id: i32,

    pub name: CryptString,
    pub deadline: NaiveDate,
    pub details: CryptString,
    pub completed: bool,
}

// course: consists of: name (cryptstring)
// topic: consists of: course_id (foreign key), name (cryptstring), details (cryptstring)
// study_goal: consists of: topic_id (foreign key), deadline (date), 
// exam: consists of: course_id (foreign key), name (cryptstring), date (date)

// todo: consists of: name (cryptstring), deadline (date), details (crypstring), completed (bool)