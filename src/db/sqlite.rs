use std::{error::Error, path::Path, sync::Arc};

use r2d2::{Pool, PooledConnection};
use r2d2_sqlite::SqliteConnectionManager;
use rusqlite::params;

use super::{DBInterface, User};

pub struct SqliteDatabase {
    pool: Arc<Pool<SqliteConnectionManager>>,
}

impl SqliteDatabase {
    /// Create a new SqliteConnectionManager (for thread safe access) with the corresponding path as file name.
    pub fn new<P: AsRef<Path>>(path: P) -> Result<Self, Box<dyn Error>> {
        // Create a connection manager for SQLite
        let manager = SqliteConnectionManager::file(path);
        let pool = Pool::new(manager)?;

        // Initialize the database
        let db = Self {
            pool: Arc::new(pool),
        };
        db.create_tables()?;

        Ok(db)
    }

    /// Get a connection from the pool
    fn get_conn(&self) -> Result<PooledConnection<SqliteConnectionManager>, r2d2::Error> {
        self.pool.get()
    }

    /// create tables in the database if they do not exist
    fn create_tables(&self) -> Result<(), Box<dyn Error>> {
        let conn = self.get_conn()?;
        // Create user table
        conn.execute(
            "CREATE TABLE IF NOT EXISTS users (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                username TEXT NOT NULL UNIQUE,
                password_hash TEXT NOT NULL,
                password_salt TEXT NOT NULL,
                created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
            )",
            [],
        )?;

        Ok(())
    }
}

impl DBInterface for SqliteDatabase {
    fn get_user_by_username(&self, username: &str) -> Result<User, Box<dyn Error>> {
        let conn = self.get_conn()?;
    
        let sql = "SELECT u.id, u.username, u.password_hash, u.password_salt, u.created_at FROM users u WHERE u.username = ?1";
        let user = conn.query_row(sql, params![username], |row| {
            Ok(User {
                id: row.get(0)?,
                username: row.get(1)?,
                password_hash: row.get(2)?,
                password_salt: row.get(3)?,
                created_at: row.get(4)?, // FIXME: Use chrono::NaiveDateTime for better date handling
            })
        })?;

        Ok(user)    
    }
}
