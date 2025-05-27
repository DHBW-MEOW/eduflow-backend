use simple_crypt_prov::SimpleCryptProv;

mod simple_crypt_prov;

/// Trait which has to be implemented for the used encrpytion method
pub trait CryptProvider {
    fn encrypt(data: &[u8], key: &[u8]) -> Vec<u8>;
    fn decrypt(data_crypt: &[u8], key: &[u8]) -> Vec<u8>;
}

pub enum CryptProviders {
    SimpleCryptProv,
}

pub fn decrypt(data_crypt: &[u8], key: &[u8], crypt_provider: &CryptProviders) -> Vec<u8> {
    match crypt_provider {
        CryptProviders::SimpleCryptProv => SimpleCryptProv::decrypt(data_crypt, key),
    }
}

pub fn encrypt(data: &[u8], key: &[u8], crypt_provider: &CryptProviders) -> Vec<u8> {
    match crypt_provider {
        CryptProviders::SimpleCryptProv => SimpleCryptProv::encrypt(data, key),
    }
}

