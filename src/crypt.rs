use std::error::Error;

use crypt_provider::CryptProviders;

pub mod crypt_provider;
pub mod crypt_types;

// Trait that has to be implemented for every data type that is encryptable
pub trait Cryptable<T> {
    fn encrypt(data: &T, key: &[u8], provider: &CryptProviders) -> Self;
    fn decrypt(&self, key: &[u8], provider: &CryptProviders) -> Result<T, Box<dyn Error>>;
}
