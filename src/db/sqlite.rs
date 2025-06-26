use std::{error::Error, path::Path, sync::Arc};

use chrono::NaiveDateTime;
use log::debug;
use r2d2::{Pool, PooledConnection};
use r2d2_sqlite::SqliteConnectionManager;
use rusqlite::{ToSql, params};

use crate::crypt::crypt_types::CryptString;

use super::{
    DBInterface, DBObjIdent, LocalTokenPWCrypt, LocalTokenRTCrypt, RemoteToken, User,
    sql_helper::{SQLGenerate, SQLValue},
};

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
                local_token BLOB NOT NULL,
                used_for TEXT NOT NULL
            )",
            [],
        )?;

        // local token table remote token encrypted (stores encrypted local tokens)
        // these tokens are encrypted with the remote token, which can be invalidated by deleting db entries in this table
        // resulting in a remote token only having access to local tokens, which have been encrypted with it.
        conn.execute(
            "CREATE TABLE IF NOT EXISTS rtcrypt_local_token (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                local_token_id INTEGER NOT NULL,
                local_token BLOB NOT NULL,
                decrypt_by_rt_id INTEGER NOT NULL
            )",
            [],
        )?;

        // remote token hashes are stored in this table, used to write access
        conn.execute(
            "CREATE TABLE IF NOT EXISTS remote_token (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                rt_hash TEXT NOT NULL,
                user_id INTEGER NOT NULL,
                valid_until TIMESTAMP NOT NULL
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

    fn new_user(&self, username: &str, password_hash: &str) -> Result<i32, Box<dyn Error>> {
        let conn = self.get_conn()?;

        let sql = "INSERT INTO user (username, password_hash) VALUES (?1, ?2)";
        conn.execute(sql, params![username, password_hash])?;

        debug!("Created new user");

        let id = conn.last_insert_rowid();
        Ok(id.try_into().expect("DB Ids exceed i32"))
    }

    fn new_local_token_pwcrypt(
        &self,
        user_id: i32,
        token_crypt: &CryptString,
        used_for: &DBObjIdent,
    ) -> Result<(), Box<dyn Error>> {
        let conn = self.get_conn()?;

        let sql =
            "INSERT INTO pwcrypt_local_token (user_id, local_token, used_for) VALUES (?1, ?2, ?3)";
        conn.execute(
            sql,
            params![user_id, token_crypt.data_crypt, used_for.db_identifier],
        )?;

        debug!("Created new user bound local token (password encrypted)");

        Ok(())
    }

    fn new_local_token_rtcrypt(
        &self,
        local_token_id: i32,
        local_token_crypt: &CryptString,
        decryptable_by_rt_id: i32,
    ) -> Result<(), Box<dyn Error>> {
        let conn = self.get_conn()?;

        let sql = "INSERT INTO rtcrypt_local_token (local_token_id, local_token, decrypt_by_rt_id) VALUES (?1, ?2, ?3)";
        conn.execute(
            sql,
            params![
                local_token_id,
                local_token_crypt.data_crypt,
                decryptable_by_rt_id
            ],
        )?;

        debug!("Created new remote token encrypted local token");

        Ok(())
    }

    fn get_local_tokens_by_user_pwcrypt(
        &self,
        user_id: i32,
    ) -> Result<Vec<LocalTokenPWCrypt>, Box<dyn Error>> {
        let conn = self.get_conn()?;
        let mut stmt = conn.prepare("SELECT lt.id, lt.user_id, lt.local_token, lt.used_for FROM pwcrypt_local_token lt WHERE lt.user_id = ?1")?;
        let local_tokens = stmt.query_map(params![user_id], |row| {
            Ok(LocalTokenPWCrypt {
                id: row.get(0)?,
                user_id: row.get(1)?,
                token_crypt: CryptString {
                    data_crypt: row.get(2)?,
                },
                //used_for: DBStructs::from(row.get::<usize, String>(3)?),
                used_for: DBObjIdent {
                    db_identifier: row.get(3)?,
                },
            })
        })?;

        let local_tokens: Vec<LocalTokenPWCrypt> = local_tokens.collect::<Result<Vec<_>, _>>()?;

        Ok(local_tokens)
    }

    fn get_local_token_by_used_for_pwcrypt(
        &self,
        user_id: i32,
        used_for: &DBObjIdent,
    ) -> Result<LocalTokenPWCrypt, Box<dyn Error>> {
        let conn = self.get_conn()?;
        let sql = "SELECT lt.id, lt.user_id, lt.local_token, lt.used_for FROM pwcrypt_local_token lt WHERE lt.user_id = ?1 AND lt.used_for = ?2";
        let local_token = conn.query_row(sql, params![user_id, used_for.db_identifier], |row| {
            Ok(LocalTokenPWCrypt {
                id: row.get(0)?,
                user_id: row.get(1)?,
                token_crypt: CryptString {
                    data_crypt: row.get(2)?,
                },
                used_for: DBObjIdent {
                    db_identifier: row.get(3)?,
                },
            })
        })?;

        Ok(local_token)
    }

    fn get_local_token_by_id_rtcrypt(
        &self,
        local_token_id: i32,
        remote_token_id: i32,
    ) -> Result<LocalTokenRTCrypt, Box<dyn Error>> {
        let conn = self.get_conn()?;
        let sql = "SELECT lt.id, lt.local_token_id, lt.local_token, lt.decrypt_by_rt_id FROM rtcrypt_local_token lt WHERE lt.local_token_id = ?1 AND lt.decrypt_by_rt_id = ?2";
        let local_token = conn.query_row(sql, params![local_token_id, remote_token_id], |row| {
            Ok(LocalTokenRTCrypt {
                id: row.get(0)?,
                local_token_id: row.get(1)?,
                local_token_crypt: CryptString {
                    data_crypt: row.get(2)?,
                },
                decryptable_by_rt_id: row.get(3)?,
            })
        })?;

        Ok(local_token)
    }

    fn new_remote_token(
        &self,
        rt_hash: &str,
        user_id: i32,
        valid_until: &NaiveDateTime,
    ) -> Result<i64, Box<dyn Error>> {
        let conn = self.get_conn()?;
        let sql = "INSERT INTO remote_token (rt_hash, user_id, valid_until) VALUES (?1, ?2, ?3)";
        conn.execute(sql, params![rt_hash, user_id, valid_until])?;

        debug!("Created new user bound remote token (hashed)");

        let id = conn.last_insert_rowid();
        Ok(id)
    }

    fn get_remote_token(&self, token_id: i32) -> Result<RemoteToken, Box<dyn Error>> {
        let conn = self.get_conn()?;
        let sql = "SELECT rt.id, rt.rt_hash, rt.user_id, rt.valid_until FROM remote_token rt WHERE rt.id = ?1";
        let remote_token = conn.query_row(sql, params![token_id], |row| {
            Ok(RemoteToken {
                id: row.get(0)?,
                rt_hash: row.get(1)?,
                user_id: row.get(2)?,
                valid_until: row.get(3)?,
            })
        })?;

        Ok(remote_token)
    }

    fn del_local_token_rtcrypt_by_rt(&self, remote_token_id: i32) -> Result<(), Box<dyn Error>> {
        let conn = self.get_conn()?;
        let sql = "DELETE FROM rtcrypt_local_token WHERE decrypt_by_rt_id = ?1";
        conn.execute(sql, params![remote_token_id])?;

        Ok(())
    }

    fn del_remote_token(&self, remote_token_id: i32) -> Result<(), Box<dyn Error>> {
        let conn = self.get_conn()?;
        let sql = "DELETE FROM remote_token WHERE id = ?1";
        conn.execute(sql, params![remote_token_id])?;

        Ok(())
    }

    // DATA OBJECTS
    /// creates and prepares a db table
    fn create_table_for_type<T: SQLGenerate>(&self) -> Result<(), Box<dyn Error>> {
        let conn = self.get_conn()?;
        let sql = T::get_db_table_create();
        conn.execute(&sql, [])?;

        Ok(())
    }

    /// creates a new db_entry, returns the resulting id
    /// params need to be a complete list of all fields in the struct of type T (order does not matter), do not include the id field (it is autoincrement).
    fn new_entry<T: SQLGenerate>(
        &self,
        params: Vec<(String, SQLValue)>,
    ) -> Result<i32, Box<dyn Error>> {
        let conn = self.get_conn()?;
        let sql = T::get_db_insert(params.iter().map(|e| &e.0).collect());
        let params: Vec<&dyn ToSql> = params
            .iter()
            .map(|param| sql_value_to_to_sql(&param.1))
            .collect();

        conn.execute(&sql, params.as_slice())?;

        let id = conn.last_insert_rowid();
        Ok(id.try_into().expect("Id value exceeding i32"))
    }

    /// selects an amount of entries and returns them
    /// params are used to select the correct entries (will be inserted at the WHERE clause)
    fn select_entries<T: SQLGenerate>(
        &self,
        params: Vec<(String, String)>,
    ) -> Result<Vec<T>, Box<dyn Error>> {
        let conn = self.get_conn()?;
        let sql = T::get_db_select(params.iter().map(|entry| &entry.0).collect());
        let mut stmt = conn.prepare(&sql)?;

        let params: Vec<&dyn ToSql> = params
            .iter()
            .map(|e| &e.1)
            .map(|param| param as &dyn ToSql)
            .collect();

        let entries = stmt.query_map(params.as_slice(), |row| T::row_to_struct(row))?;

        let local_tokens: Vec<T> = entries.collect::<Result<Vec<_>, _>>()?;
        Ok(local_tokens)
    }

    /// updates entries and returns ok on success
    /// params are the params which should be changed
    /// where_params are the params which will be filtered on in the WHERE clause
    fn update_entry<T: SQLGenerate>(
        &self,
        params: Vec<(String, SQLValue)>,
        where_params: Vec<(String, SQLValue)>,
    ) -> Result<(), Box<dyn Error>> {
        let conn = self.get_conn()?;
        let sql = T::get_db_update(
            params.iter().map(|entry| &entry.0).collect(),
            where_params.iter().map(|entry| &entry.0).collect(),
        );

        let params: Vec<&dyn ToSql> = params
            .iter()
            .chain(where_params.iter())
            .map(|e| &e.1)
            .map(|param| sql_value_to_to_sql(param))
            .collect();

        conn.execute(&sql, params.as_slice())?;

        Ok(())
    }

    /// deletes an entry and returns ok on success
    /// params is the WHERE clause, which select what entry to delete
    fn delete_entry<T: SQLGenerate>(
        &self,
        params: Vec<(String, SQLValue)>,
    ) -> Result<(), Box<dyn Error>> {
        let conn = self.get_conn()?;
        let sql = T::get_db_delete(params.iter().map(|e| &e.0).collect());

        let params: Vec<&dyn ToSql> = params
            .iter()
            .map(|e| &e.1)
            .map(|param| sql_value_to_to_sql(param))
            .collect();

        conn.execute(&sql, params.as_slice())?;

        Ok(())
    }
}

fn sql_value_to_to_sql(param: &SQLValue) -> &dyn ToSql {
    match param {
        super::sql_helper::SQLValue::Text(s) => s as &dyn ToSql,
        super::sql_helper::SQLValue::Int32(i) => i as &dyn ToSql,
        super::sql_helper::SQLValue::Blob(items) => items as &dyn ToSql,
        super::sql_helper::SQLValue::Float64(f) => f as &dyn ToSql,
        super::sql_helper::SQLValue::Date(d) => d as &dyn ToSql,
        super::sql_helper::SQLValue::Bool(b) => b as &dyn ToSql,
    }
}
