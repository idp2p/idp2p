use std::io::Read;

use crate::random::create_random;

use super::{error::Idp2pMultiError, AES256_CODE, AESGCM_CODE};
use unsigned_varint::{encode as varint_encode, io::read_u64};

pub enum Idp2pEncryptionMethod {
    AesGcm { iv: Vec<u8> },
}

impl Idp2pEncryptionMethod {
    pub fn new_aes_gcm() -> Self {
        let iv: [u8; 12] = create_random();
        Self::AesGcm{iv: iv.to_vec()}
    }

    pub fn encrypt(&self, key: &[u8], msg: &[u8]) -> Result<Vec<u8>, Idp2pMultiError> {
        match self {
            Self::AesGcm(iv) => {
                use aes_gcm::{
                    aead::{generic_array::GenericArray, Aead, NewAead},
                    Aes256Gcm,
                };
                let nonce = GenericArray::from_slice(&iv[..12]);
                let aead = Aes256Gcm::new(GenericArray::from_slice(key));
                let cipher = aead
                    .encrypt(&nonce, msg)
                    .map_err(|_| Idp2pMultiError::EncryptionError)?;
                Ok(cipher)
            }
        }
    }

    pub fn decrypt(&self, key: &[u8], cipher: &[u8]) -> Result<Vec<u8>, Idp2pMultiError> {
        match self {
            Self::AesGcm(iv) => {
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

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn enc_dec_test() -> Result<(), Idp2pMultiError> {
        let key = Idp2pEncryptionAlg::new_aes_gcm();
        let key_bytes = key.to_bytes();
        let _ = Idp2pEncryptionAlg::from_bytes(key_bytes)?;
        Ok(())
    }
}
