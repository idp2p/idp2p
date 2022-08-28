pub mod x25519;
use std::io::Read;

use self::x25519::{X25519Keypair, X25519PublicKey};
use super::error::Idp2pMultiError;
use sha2::{Digest, Sha256};
use unsigned_varint::{encode as varint_encode, io::read_u64};

#[derive(PartialEq, Clone, Debug)]
pub enum AgreementKeyCode {
    X25519 = 0xec,
    Kyber768 = 0x6b768,
}

const X25519_MULTICODE: u64 = AgreementKeyCode::X25519 as u64;
const KYBER768_MULTICODE: u64 = AgreementKeyCode::Kyber768 as u64;
impl TryFrom<u64> for AgreementKeyCode {
    type Error = Idp2pMultiError;

    fn try_from(value: u64) -> Result<Self, Self::Error> {
        match value {
            X25519_MULTICODE => Ok(Self::X25519),
            KYBER768_MULTICODE => Ok(Self::Kyber768),
            _ => Err(Idp2pMultiError::InvalidKeyCode),
        }
    }
}
impl AgreementKeyCode {
    pub fn pub_size(&self) -> u64 {
        match &self {
            AgreementKeyCode::X25519 => 32,
            AgreementKeyCode::Kyber768 => 1184,
        }
    }
}
pub struct AgreementShared {
    // Shared secret
    pub secret: Vec<u8>,
    // Ephemeral key or ciphertext
    pub data: Vec<u8>,
}

pub trait AgreementPublicBehaviour {
    fn pub_bytes(&self) -> Vec<u8>;
    fn create_data(&self) -> Result<AgreementShared, Idp2pMultiError>;
}

pub trait AgreementSecretBehaviour {
    type PublicKeyType;
    fn priv_bytes(&self) -> Vec<u8>;
    fn to_public_key(&self) -> Self::PublicKeyType;
    fn resolve_shared_secret(&self, data: &[u8]) -> Result<Vec<u8>, Idp2pMultiError>;
}

#[derive(PartialEq, Clone, Debug)]
pub enum Idp2pAgreementPublicKey {
    X25519(X25519PublicKey),
    Kyber768(),
}

#[derive(Debug, Clone, PartialEq)]
pub enum Idp2pAgreementKeypair {
    X25519(X25519Keypair),
    Kyber768(),
}

impl Idp2pAgreementKeypair {
    pub fn to_public_key(&self) -> Idp2pAgreementPublicKey {
        match self {
            Self::X25519(xsecret) => Idp2pAgreementPublicKey::X25519(xsecret.to_public_key()),
            Self::Kyber768() => todo!(),
        }
    }

    pub fn resolve_shared_key(&self, data: &[u8]) -> Result<Vec<u8>, Idp2pMultiError> {
        match self {
            Self::X25519(xsecret) => xsecret.resolve_shared_secret(data),
            Self::Kyber768() => todo!(),
        }
    }
}
impl Idp2pAgreementPublicKey {
    pub fn new(code: u64, bytes: &[u8]) -> Result<Self, Idp2pMultiError> {
        match code {
            X25519_MULTICODE => Ok(Self::X25519(X25519PublicKey(bytes.try_into()?))),
            KYBER768_MULTICODE => Ok(Self::X25519(X25519PublicKey(bytes.try_into()?))),
            _ => Err(Idp2pMultiError::InvalidKeyCode),
        }
    }

    pub fn from_bytes(bytes: &[u8]) -> Result<Self, Idp2pMultiError> {
        let mut r = bytes;
        let code: AgreementKeyCode = read_u64(&mut r)?.try_into()?;
        match code {
            AgreementKeyCode::X25519 => {
                let mut public = [0u8; 32];
                r.read_exact(&mut public)?;
                return Ok(Self::X25519(X25519PublicKey::from_bytes(public)));
            }
            AgreementKeyCode::Kyber768 => todo!(),
        }
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        let (code, bytes) = match &self {
            Self::X25519(public) => (X25519_MULTICODE, public.pub_bytes()),
            Self::Kyber768() => todo!(),
        };
        let mut code_buf = varint_encode::u64_buffer();
        let code = varint_encode::u64(code, &mut code_buf);
        [code, &bytes].concat()
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
            .chain_update(&self.to_bytes())
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
        //let key = AgreementPublicKey::new(0xec, &bytes)?;
        //let decoded_key = AgreementPublicKey::from_bytes(&key.to_bytes())?;
        //matches!(decoded_key, AgreementPublicKey::X25519(X25519PublicKey(public)) if public == bytes);
        Ok(())
    }
}
