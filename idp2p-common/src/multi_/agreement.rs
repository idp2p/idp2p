pub mod x25519;
use std::io::Read;

use self::x25519::X25519PublicKey;
use super::error::Idp2pMultiError;
use sha2::{Digest, Sha256};
use unsigned_varint::{encode as varint_encode, io::read_u64};
const X25519_MULTICODE: u64 = 0xec;
const KYBER768_MULTICODE: u64 = 0x6b768;

pub struct AgreementShared {
    // Shared secret
    pub secret: Vec<u8>,
    // Ephemeral key or ciphertext
    pub data: Vec<u8>,
}

pub trait AgreementPublicKey {
    fn pub_bytes(&self) -> Vec<u8>;
    fn create_data(&self) -> Result<AgreementShared, Idp2pMultiError>;
}

#[derive(PartialEq, Clone, Debug)]
pub enum Idp2pAgreementPublicKey {
    X25519(X25519PublicKey),
    Kyber768(),
}

impl Idp2pAgreementPublicKey {
    pub fn new(code: u64, bytes: &[u8]) -> Result<Self, Idp2pMultiError> {
        todo!()
    }
    pub fn decode(bytes: &[u8]) -> Result<Self, Idp2pMultiError> {
        let mut r = bytes;
        let code = read_u64(&mut r)?.try_into()?;
        let size = read_u64(&mut r)?;
        let mut public = vec![0u8; size as usize];
        r.read_exact(&mut public)?;
        match code {
            X25519_MULTICODE => Ok(Self::X25519(X25519PublicKey((&*public).try_into()?))),
            KYBER768_MULTICODE => Ok(Self::X25519(X25519PublicKey((&*public).try_into()?))),
            _ => Err(Idp2pMultiError::InvalidKeyCode),
        }
    }

    pub fn encode(&self) -> Vec<u8> {
        let (code, size, bytes) = match &self {
            Self::X25519(public) => (X25519_MULTICODE, 32u64, public.pub_bytes()),
            Self::Kyber768() => todo!(),
        };
        let mut code_buf = varint_encode::u64_buffer();
        let code = varint_encode::u64(code, &mut code_buf);
        let mut size_buf = varint_encode::u64_buffer();
        let size = varint_encode::u64(size, &mut size_buf);
        [code, size, &bytes].concat()
    }

    // Create a new shared secret and data for the public key
    pub fn create_shared(&self) -> Result<AgreementShared, Idp2pMultiError> {
        match &self {
            Self::X25519(public) => public.create_data(),
            Self::Kyber768() => todo!(),
        }
    }

    pub fn generate_id(&self) -> [u8; 16] {
        let h = Sha256::new()
            .chain_update(&self.encode())
            .finalize()
            .to_vec();
        h[0..16].try_into().expect("Conversion failed")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn enc_dec_test() -> Result<(), Idp2pMultiError> {
        let bytes = [0u8; 32];
        let key = Idp2pAgreementPublicKey::new(0xec, &bytes)?;
        let decoded_key = Idp2pAgreementPublicKey::decode(&key.encode())?;
        matches!(decoded_key, Idp2pAgreementPublicKey::X25519(X25519PublicKey(public)) if public == bytes);
        Ok(())
    }
}
