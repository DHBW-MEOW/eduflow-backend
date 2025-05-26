pub mod simple_crypt_prov;

/// Trait which has to be implemented for the used encrpytion method
pub trait CryptProvider {
    fn encrypt(&self, data: &[u8], key: &[u8]) -> Vec<u8>;
    fn decrypt(&self, data_crypt: &[u8], key: &[u8]) -> Vec<u8>;
}