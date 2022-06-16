use crate::random::create_random;

use super::error::Idp2pMultiError;

pub struct EncryptedContent {
    pub enc_alg: String,
    pub initial_vector: Vec<u8>,
    pub cipher: Vec<u8>,
}
pub enum Idp2pEncryptionKey {
    AesGcm(Vec<u8>),
}

impl Idp2pEncryptionKey {
    pub fn encrypt(&self, msg: &[u8]) -> Result<EncryptedContent, Idp2pMultiError> {
        match self {
            Idp2pEncryptionKey::AesGcm(key) => {
                use aes_gcm::{
                    aead::{generic_array::GenericArray, Aead, NewAead},
                    Aes256Gcm,
                };
                let iv: [u8; 12] = create_random();
                let nonce = GenericArray::from_slice(&iv[..12]);
                let aead = Aes256Gcm::new(GenericArray::from_slice(key));
                let cipher = aead
                    .encrypt(&nonce, msg)
                    .map_err(|_| Idp2pMultiError::EncryptionError)?;
                Ok(EncryptedContent {
                    enc_alg: "A256CBC_HS512".to_string(),
                    initial_vector: iv.to_vec(),
                    cipher: cipher,
                })
            }
        }
    }

    pub fn decrypt(&self, iv: &[u8], cipher: &[u8]) -> Result<Vec<u8>, Idp2pMultiError> {
        match self {
            Idp2pEncryptionKey::AesGcm(key) => {
                use aes_gcm::{
                    aead::{generic_array::GenericArray, Aead, NewAead},
                    Aes256Gcm,
                };
                let nonce = GenericArray::from_slice(iv);
                let aead = Aes256Gcm::new(GenericArray::from_slice(key));
                let body = aead
                    .decrypt(nonce, cipher)
                    .map_err(|_| Idp2pMultiError::DecryptionError)?;
                Ok(body)
            }
        }
    }
}
