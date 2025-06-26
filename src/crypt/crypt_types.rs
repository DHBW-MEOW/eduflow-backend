use std::error::Error;

use rusqlite::types::FromSql;

use super::{
    Cryptable,
    crypt_provider::{CryptProviders, decrypt, encrypt},
};

/// Encrypted type of String
#[derive(Debug)]
pub struct CryptString {
    pub data_crypt: Vec<u8>,
}

impl Cryptable<String> for CryptString {
    fn encrypt(data: &String, key: &[u8], provider: &CryptProviders) -> CryptString {
        Self {
            data_crypt: encrypt(data.as_bytes(), key, provider).expect("Encryption failure!"),
        }
    }

    fn decrypt(&self, key: &[u8], provider: &CryptProviders) -> Result<String, Box<dyn Error>> {
        let data = decrypt(&self.data_crypt, key, provider);

        Ok(String::from_utf8(data?)?)
    }
}

impl FromSql for CryptString {
    fn column_result(value: rusqlite::types::ValueRef<'_>) -> rusqlite::types::FromSqlResult<Self> {
        let blob = value.as_blob()?;
        Ok(CryptString {
            data_crypt: blob.to_vec(),
        })
    }
}

/// Encrypted type of i32
#[derive(Debug)]
pub struct CryptI32 {
    pub data_crypt: Vec<u8>,
}

impl Cryptable<i32> for CryptI32 {
    fn encrypt(data: &i32, key: &[u8], provider: &CryptProviders) -> Self {
        Self {
            data_crypt: encrypt(&data.to_be_bytes(), key, provider).expect("Encryption failure!"),
        }
    }

    fn decrypt(&self, key: &[u8], provider: &CryptProviders) -> Result<i32, Box<dyn Error>> {
        let data = decrypt(&self.data_crypt, key, provider);

        let arr: [u8; 4] = data?
            .as_slice()
            .try_into()
            .expect("DB data corrupted, tried to decrypt but got wrong format.");
        Ok(i32::from_be_bytes(arr))
    }
}

impl FromSql for CryptI32 {
    fn column_result(value: rusqlite::types::ValueRef<'_>) -> rusqlite::types::FromSqlResult<Self> {
        let blob = value.as_blob()?;
        Ok(CryptI32 {
            data_crypt: blob.to_vec(),
        })
    }
}
