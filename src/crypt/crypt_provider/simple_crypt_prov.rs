use std::error::Error;

use simple_crypt::{decrypt, encrypt};

use super::CryptProvider;

/// Crypt provider using simple_crypt crate
pub struct SimpleCryptProv {}

impl CryptProvider for SimpleCryptProv {
    fn encrypt(data: &[u8], key: &[u8]) -> Result<Vec<u8>, Box<dyn Error>> {
        Ok(encrypt(data, key)?)
    }

    fn decrypt(data_crypt: &[u8], key: &[u8]) -> Result<Vec<u8>, Box<dyn Error>> {
        Ok(decrypt(data_crypt, key)?)
    }
}

unsafe impl Send for SimpleCryptProv {}
unsafe impl Sync for SimpleCryptProv {}
