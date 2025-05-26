use std::error::Error;

use crate::crypt::crypt_types::{CryptI32, CryptString};


pub mod sqlite;

/// Database interface trait that defines the methods for database operations.
pub trait DBInterface {
    // user related
    /// create a new user
    fn new_user(&self, username: &str, password_hash: &str) -> Result<(), Box<dyn Error>>;
    /// Get a user by their username.
    fn get_user_by_username(&self, username: &str) -> Result<User, Box<dyn Error>>;

    // dummy related
    fn new_dummy(&self, name: &str, secret_number: &CryptI32, secret_text: &CryptString) -> Result<(), Box<dyn Error>>;

    fn get_dummy(&self, id: i32) -> Result<TestDummy, Box<dyn Error>>;
}

// structs, which are stored inside of the database
#[derive(Debug)]
pub struct User {
    pub id: i32,
    pub username: String,
    pub password_hash: String,
    pub created_at: String, // FIXME: Use chrono::NaiveDateTime for better date handling
}

/// just a testing struct so we can confirm functionallity
#[derive(Debug)]
pub struct TestDummy {
    pub id: i32,
    pub name: String,
    pub secret_number: CryptI32,
    pub secret_text: CryptString,
}