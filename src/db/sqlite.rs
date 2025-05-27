use std::{error::Error, path::Path, sync::Arc};

use log::debug;
use r2d2::{Pool, PooledConnection};
use r2d2_sqlite::SqliteConnectionManager;
use rusqlite::params;

use crate::crypt::crypt_types::{CryptI32, CryptString};

use super::{DBInterface, LocalTokenPWCrypt, LocalTokenRTCrypt, TestDummy, User};

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
        db.create_auth_tables()?;
        db.create_data_tables()?;

        Ok(db)
    }

    /// Get a connection from the pool
    fn get_conn(&self) -> Result<PooledConnection<SqliteConnectionManager>, r2d2::Error> {
        self.pool.get()
    }

    /// create tables in the database if they do not exist
    fn create_auth_tables(&self) -> Result<(), Box<dyn Error>> {
        let conn = self.get_conn()?;
        // Create user table
        conn.execute(
            "CREATE TABLE IF NOT EXISTS user (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                username TEXT NOT NULL UNIQUE,
                password_hash TEXT NOT NULL,
                created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
            )",
            [],
        )?;

        // local token table pw encrypted (stores encrypted local tokens)
        // these tokens are encrypted with the users password
        conn.execute(
            "CREATE TABLE IF NOT EXISTS pwcrypt_local_token (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                user_id INTEGER NOT NULL,
                local_token BLOB NOT NULL
            )", 
            [],
        )?;

        // local token table remote token encrypted (stores encrypted local tokens)
        // these tokens are encrypted with the remote token, which can be invalidated by deleting db entries in this table
        // remote_token_hash stores the hash of the remote token, resulting in a remote token only having access to local tokens, which have been encrypted with it.
        conn.execute(
            "CREATE TABLE IF NOT EXISTS rtcrypt_local_token (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                local_token_id INTEGER NOT NULL,
                local_token BLOB NOT NULL,
                remote_token_hash TEXT NOT NULL,
                valid_until TIMESTAMP NOT NULL
            )", 
            [],
        )?;
        

        Ok(())
    }

    fn create_data_tables(&self) -> Result<(), Box<dyn Error>> {
        let conn = self.get_conn()?;
        // dummy table
        // decryptable by is the id of the local token that can decrypt the fields
        conn.execute(
            "CREATE TABLE IF NOT EXISTS dummy (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                name TEXT NOT NULL,
                number BLOB NOT NULL,
                text BLOB NOT NULL,
                decryptable_by INTEGER NOT NULL
            )",
        [],
        )?;

        Ok(())
    }
}

impl DBInterface for SqliteDatabase {
    // AUTH OBJECTS

    // user related
    fn get_user_by_username(&self, username: &str) -> Result<User, Box<dyn Error>> {
        let conn = self.get_conn()?;
    
        let sql = "SELECT u.id, u.username, u.password_hash, u.created_at FROM user u WHERE u.username = ?1";
        let user = conn.query_row(sql, params![username], |row| {
            Ok(User {
                id: row.get(0)?,
                username: row.get(1)?,
                password_hash: row.get(2)?,
                created_at: row.get(3)?,
            })
        })?;

        Ok(user)    
    }
    
    fn new_user(&self, username: &str, password_hash: &str) -> Result<(), Box<dyn Error>> {
        let conn = self.get_conn()?;

        let sql = "INSERT INTO user (username, password_hash) VALUES (?1, ?2)";
        conn.execute(sql, params![username, password_hash])?;
        
        debug!("Created new user");
        Ok(())
    }

    fn new_local_token_pwcrypt(&self, user_id: i32, token_crypt: CryptString) -> Result<(), Box<dyn Error>> {
        let conn = self.get_conn()?;

        let sql = "INSERT INTO pwcrypt_local_token (user_id, local_token) VALUES (?1, ?2)";
        conn.execute(sql, params![user_id, token_crypt.data_crypt])?;

        debug!("Created new user bound local token (password encrypted)");

        Ok(())
    }
    
    fn new_local_token_rtcrypt(&self, local_token_id: i32, local_token_crypt: &CryptString, remote_token_hash: &str, valid_until: &chrono::NaiveDateTime) -> Result<(), Box<dyn Error>> {
        let conn = self.get_conn()?;

        let sql = "INSERT INTO rtcrypt_local_token (local_token_id, local_token, remote_token_hash, valid_until) VALUES (?1, ?2, ?3, ?4)";
        conn.execute(sql, params![local_token_id, local_token_crypt.data_crypt, remote_token_hash, valid_until])?;

        debug!("Created new remote token encrypted local token");

        Ok(())
    }
    
    fn get_local_token_by_user_pwcrypt(&self, user_id: i32) -> Result<Vec<LocalTokenPWCrypt>, Box<dyn Error>> {
        let conn = self.get_conn()?;
        let mut stmt = conn.prepare("SELECT lt.id, lt.user_id, lt.local_token FROM pwcrypt_local_token lt WHERE lt.user_id = ?1")?;
        let local_tokens = stmt.query_map(params![user_id], |row| {
            Ok(LocalTokenPWCrypt {
                id: row.get(0)?,
                user_id: row.get(1)?,
                token_crypt: CryptString { data_crypt: row.get(2)? },
            })
        })?;

        let local_tokens: Vec<LocalTokenPWCrypt> = local_tokens.collect::<Result<Vec<_>, _>>()?;

        Ok(local_tokens)
    }
    
    fn get_local_token_by_id_pwcrypt(&self, local_token_id: i32) -> Result<LocalTokenPWCrypt, Box<dyn Error>> {
        let conn = self.get_conn()?;
        let sql = "SELECT lt.id, lt.user_id, lt.local_token FROM pwcrypt_local_token lt WHERE lt.id = ?1";
        let local_token = conn.query_row(sql, params![local_token_id], |row| {
            Ok(LocalTokenPWCrypt {
                id: row.get(0)?,
                user_id: row.get(1)?,
                token_crypt: CryptString { data_crypt: row.get(2)? },
            })
        })?;

        Ok(local_token)
    }
    
    fn get_local_tokens_by_rthash(&self, remote_token_hash: &str) -> Result<Vec<LocalTokenRTCrypt>, Box<dyn Error>> {
        let conn = self.get_conn()?;
        let mut stmt = conn.prepare("SELECT lt.id, lt.local_token_id, lt.local_token, lt.remote_token_hash, lt.valid_until FROM rtcrypt_local_token lt WHERE lt.remote_token_hash = ?1")?;
        let local_tokens = stmt.query_map(params![remote_token_hash], |row| {
            Ok(LocalTokenRTCrypt {
                id: row.get(0)?,
                local_token_id: row.get(1)?,
                local_token_crypt: CryptString { data_crypt: row.get(2)? },
                remote_token_hash: row.get(3)?,
                valid_until: row.get(4)?,
            })
        })?;

        let local_tokens: Vec<LocalTokenRTCrypt> = local_tokens.collect::<Result<Vec<_>, _>>()?;

        Ok(local_tokens)
    }
    
    fn get_local_token_by_id_rtcrypt(&self, local_token_id: i32) -> Result<LocalTokenRTCrypt, Box<dyn Error>> {
        let conn = self.get_conn()?;
        let sql = "SELECT lt.id, lt.local_token_id, lt.local_token, lt.remote_token_hash, lt.valid_until FROM rtcrypt_local_token lt WHERE lt.local_token_id = ?1";
        let local_token = conn.query_row(sql, params![local_token_id], |row| {
            Ok(LocalTokenRTCrypt {
                id: row.get(0)?,
                local_token_id: row.get(1)?,
                local_token_crypt: CryptString { data_crypt: row.get(2)? },
                remote_token_hash: row.get(3)?,
                valid_until: row.get(4)?,
            })
        })?;

        Ok(local_token)
    }


    // DATA OBJECTS
    
    // dummy related
    fn new_dummy(&self, name: &str, number: &CryptI32, text: &CryptString, decryptable_by: i32) -> Result<(), Box<dyn Error>> {
        let conn = self.get_conn()?;

        let sql =  "INSERT INTO dummy (name, number, text, decryptable_by) VALUES (?1, ?2, ?3, ?4)";
        conn.execute(sql, params![name, number.data_crypt, text.data_crypt, decryptable_by])?;

        Ok(())
    }
    
    fn get_dummy(&self, id: i32) -> Result<super::TestDummy, Box<dyn Error>> {
        let conn = self.get_conn()?;
        let sql = "SELECT d.id, d.name, d.number, d.text d.decryptable_by FROM dummy d WHERE d.id = ?1";
        let dummy = conn.query_row(sql, params![id], |row| {
            Ok(TestDummy{
                id: row.get(0)?,
                name: row.get(1)?,
                secret_number: CryptI32 { data_crypt: row.get(2)? },
                secret_text: CryptString { data_crypt: row.get(3)? },
                decryptable_by: row.get(4)?,
            })
        })?;

        Ok(dummy)
    }
    
    
}
