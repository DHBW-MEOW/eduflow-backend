use std::error::Error;

use crypt_provider::CryptProvider;

pub mod crypt_provider;
pub mod crypt_types;

// Trait that has to be implemented for every data type that is encryptable
pub trait Cryptable<T> {
    fn encrypt<C: CryptProvider>(data: &T, key: &[u8], provider: &C) -> Self;
    fn decrypt<C: CryptProvider>(&self, key: &[u8], provider: &C) -> Result<T, Box<dyn Error>>;
}
