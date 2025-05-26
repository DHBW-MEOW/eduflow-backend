use std::error::Error;

use super::{crypt_provider::CryptProvider, Cryptable};

/// Encrypted type of String
#[derive(Debug)]
pub struct CryptString {
    pub data_crypt: Vec<u8>
}

impl Cryptable<String> for CryptString {
    fn encrypt<C: CryptProvider>(data: &String, key: &[u8], provider: &C) -> CryptString {
        Self { data_crypt: provider.encrypt(data.as_bytes(), key) }
    }

    fn decrypt<C: CryptProvider>(&self, key: &[u8], provider: &C) -> Result<String, Box<dyn Error>> {
        let data = provider.decrypt(&self.data_crypt, key);

        Ok(String::from_utf8(data)?)
    }
}

/// Encrypted type of i32
#[derive(Debug)]
pub struct CryptI32 {
    pub data_crypt: Vec<u8>
}

impl Cryptable<i32> for CryptI32 {
    fn encrypt<C: CryptProvider>(data: &i32, key: &[u8], provider: &C) -> Self {
        Self { data_crypt: provider.encrypt(&data.to_be_bytes(), key) }
    }

    fn decrypt<C: CryptProvider>(&self, key: &[u8], provider: &C) -> Result<i32, Box<dyn Error>> {
        let data = provider.decrypt(&self.data_crypt, key);

        let arr: [u8; 4] = data.as_slice().try_into().expect("DB data corrupted, tried to decrypt but got wrong format.");
        Ok(i32::from_be_bytes(arr))
    }
}