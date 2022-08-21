use std::io::Read;

use crate::random::create_random;

use super::{error::Idp2pMultiError, AES256_CODE, AESGCM_CODE};
use unsigned_varint::{encode as varint_encode, io::read_u64};

pub enum Idp2pEncryptionKeyCode {
    Aes256 = 0xec,
}

pub enum Idp2pEncryptionCode {
    AesGcm = 0xec,
}
// Change Alg to Method
pub enum Idp2pEncryptionAlg {
    AesGcm (Vec<u8>),
}

impl Idp2pEncryptionAlg {
    pub fn new_aes_gcm() -> Self {
        let iv: [u8; 12] = create_random();
        Self::AesGcm(iv.to_vec())
    }

    pub fn from_bytes<T: AsRef<[u8]>>(bytes: T) -> Result<Self, Idp2pMultiError> {
        let mut r = bytes.as_ref();
        let enc_key_type = read_u64(&mut r)?;
        let enc_type = read_u64(&mut r)?;
        match enc_key_type {
            AES256_CODE => {
                match enc_type{
                    AESGCM_CODE => {
                        let mut iv_bytes = [0u8; 12];
                        r.read_exact(&mut iv_bytes)?;
                        Ok(Self::AesGcm(iv_bytes.to_vec()))
                    },
                    _ => Err(Idp2pMultiError::InvalidKeyCode)
                }
            }
            _ => Err(Idp2pMultiError::InvalidKeyCode),
        }
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        match &self {
            Self::AesGcm (iv) => {
                let mut enc_key_type_buf = varint_encode::u64_buffer();
                let enc_key_type = varint_encode::u64(AES256_CODE, &mut enc_key_type_buf);
                let mut enc_type_buf = varint_encode::u64_buffer();
                let enc_type = varint_encode::u64(AESGCM_CODE, &mut enc_type_buf);
                [enc_key_type, enc_type, &*iv].concat()
            }
        }
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