use std::error::Error;


pub mod sqlite;

/// Database interface trait that defines the methods for database operations.
pub trait DBInterface {
    /// create a new user
    fn new_user(&self, username: &str, password_hash: &str) -> Result<(), Box<dyn Error>>;
    /// Get a user by their username.
    fn get_user_by_username(&self, username: &str) -> Result<User, Box<dyn Error>>;
}

// structs, which are stored inside of the database
#[derive(Debug)]
pub struct User {
    pub id: i32,
    pub username: String,
    pub password_hash: String,
    pub created_at: String, // FIXME: Use chrono::NaiveDateTime for better date handling
}