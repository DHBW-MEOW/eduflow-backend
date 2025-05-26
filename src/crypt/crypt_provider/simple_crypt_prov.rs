use simple_crypt::{decrypt, encrypt};

use super::CryptProvider;

/// Crypt provider using simple_crypt crate
pub struct SimpleCryptProv {}

impl CryptProvider for SimpleCryptProv {
    fn encrypt(&self, data: &[u8], key: &[u8]) -> Vec<u8> {
        encrypt(data, key).expect("fail") // FIXME: remove expect 
    }

    fn decrypt(&self, data_crypt: &[u8], key: &[u8]) -> Vec<u8> {
        decrypt(data_crypt, key).expect("decrypt fail") // FIXME: remove expect -> panics on wrong passwd
    }
}