use std::error::Error;

use simple_crypt_prov::SimpleCryptProv;

mod simple_crypt_prov;

/// Trait which has to be implemented for the used encrpytion method
pub trait CryptProvider {
    fn encrypt(data: &[u8], key: &[u8]) -> Result<Vec<u8>, Box<dyn Error>>;
    fn decrypt(data_crypt: &[u8], key: &[u8]) -> Result<Vec<u8>, Box<dyn Error>>;
}

/// enum of all possible cryptprovider, and corresponding functions to map the enum to the actual functions
pub enum CryptProviders {
    SimpleCryptProv,
}

pub fn decrypt(data_crypt: &[u8], key: &[u8], crypt_provider: &CryptProviders) -> Result<Vec<u8>, Box<dyn Error>> {
    match crypt_provider {
        CryptProviders::SimpleCryptProv => SimpleCryptProv::decrypt(data_crypt, key),
    }
}

pub fn encrypt(data: &[u8], key: &[u8], crypt_provider: &CryptProviders) -> Result<Vec<u8>, Box<dyn Error>> {
    match crypt_provider {
        CryptProviders::SimpleCryptProv => SimpleCryptProv::encrypt(data, key),
    }
}

