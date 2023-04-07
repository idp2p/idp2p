pub mod kyber768;
pub mod x25519;
use std::io::Read;

use self::{
    kyber768::{Kyber768Keypair, Kyber768PublicKey},
    x25519::{X25519Keypair, X25519PublicKey},
};
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
    /// Shared secret
    pub secret: Vec<u8>,
    /// Ephemeral key or ciphertext
    pub data: Vec<u8>,
}

pub trait AgreementPublicBehaviour {
    fn as_bytes<'a>(&'a self) -> &'a [u8];
    fn create_shared(&self) -> Result<AgreementShared, Idp2pMultiError>;
}

pub trait AgreementSecretBehaviour {
    type PublicKeyType;
    fn priv_as_bytes<'a>(&'a self) -> &'a [u8];
    fn to_public_key(&self) -> Self::PublicKeyType;
    fn resolve_shared_secret(&self, data: &[u8]) -> Result<Vec<u8>, Idp2pMultiError>;
}

#[derive(PartialEq, Clone, Debug)]
pub enum Idp2pAgreementPublicKey {
    X25519(X25519PublicKey),
    Kyber768(Kyber768PublicKey),
}

#[derive(Debug, Clone, PartialEq)]
pub enum Idp2pAgreementKeypair {
    X25519(X25519Keypair),
    Kyber768(Kyber768Keypair),
}

impl Idp2pAgreementKeypair {
    pub fn new_x25519(secret: [u8; 32]) -> Self {
        Self::X25519(X25519Keypair::from_secret_bytes(secret))
    }

    pub fn to_public_key(&self) -> Idp2pAgreementPublicKey {
        match self {
            Self::X25519(keypair) => Idp2pAgreementPublicKey::X25519(keypair.to_public_key()),
            Self::Kyber768(keypair) => Idp2pAgreementPublicKey::Kyber768(keypair.to_public_key()),
        }
    }

    pub fn resolve_shared_key(&self, data: &[u8]) -> Result<Vec<u8>, Idp2pMultiError> {
        match self {
            Self::X25519(keypair) => keypair.resolve_shared_secret(data),
            Self::Kyber768(keypair) => keypair.resolve_shared_secret(data),
        }
    }
}
impl Idp2pAgreementPublicKey {
    pub fn new(code: u64, bytes: &[u8]) -> Result<Self, Idp2pMultiError> {
        match code {
            X25519_MULTICODE => Ok(Self::X25519(X25519PublicKey(bytes.try_into()?))),
            KYBER768_MULTICODE => Ok(Self::Kyber768(Kyber768PublicKey(bytes.try_into()?))),
            _ => Err(Idp2pMultiError::InvalidKeyCode),
        }
    }

    pub fn from_multi_bytes(bytes: &[u8]) -> Result<Self, Idp2pMultiError> {
        let mut r = bytes;
        let code: AgreementKeyCode = read_u64(&mut r)?.try_into()?;
        match code {
            AgreementKeyCode::X25519 => Ok(Self::X25519((r).try_into()?)),
            AgreementKeyCode::Kyber768 => Ok(Self::Kyber768((r).try_into()?)),
        }
    }

    pub fn to_multi_bytes(&self) -> Vec<u8> {
        let (code, bytes) = match &self {
            Self::X25519(public) => (X25519_MULTICODE, public.as_bytes()),
            Self::Kyber768(public) => (KYBER768_MULTICODE, public.as_bytes()),
        };
        let mut code_buf = varint_encode::u64_buffer();
        let code = varint_encode::u64(code, &mut code_buf);
        [code, bytes].concat()
    }

    pub fn as_bytes<'a>(&'a self) -> &'a [u8] {
        match &self {
            Self::X25519(public) => public.as_bytes(),
            Self::Kyber768(public) => public.as_bytes(),
        }
    }

    // Create a new shared secret and data for the public key
    pub fn create_shared(&self) -> Result<AgreementShared, Idp2pMultiError> {
        match &self {
            Self::X25519(public) => public.create_shared(),
            Self::Kyber768(public) => public.create_shared(),
        }
    }

    pub fn generate_id(&self) -> [u8; 16] {
        let h = Sha256::new()
            .chain_update(self.as_bytes())
            .finalize()
            .to_vec();
        h[0..16].try_into().expect("Conversion failed")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn create_resolve_shared_test() -> Result<(), Idp2pMultiError> {
        let alice_keypair = Idp2pAgreementKeypair::new_x25519([0u8; 32]);
        let shared_for_alice = alice_keypair.to_public_key().create_shared()?;
        let shared_secret = alice_keypair.resolve_shared_key(&shared_for_alice.data)?;
        assert_eq!(shared_for_alice.secret, shared_secret);
        Ok(())
    }

    #[test]
    fn kyber_create_resolve_shared_test() -> Result<(), Idp2pMultiError> {
        let alice_keypair = Idp2pAgreementKeypair::Kyber768(Kyber768Keypair::generate());
        let shared_for_alice = alice_keypair.to_public_key().create_shared()?;
        let shared_secret = alice_keypair.resolve_shared_key(&shared_for_alice.data)?;
        dbg!(&shared_secret);
        assert_eq!(shared_for_alice.secret, shared_secret);
        Ok(())
    }

    #[test]
    fn enc_dec_test() -> Result<(), Idp2pMultiError> {
        let key = Idp2pAgreementPublicKey::new(
            AgreementKeyCode::Kyber768 as u64,
            &[0u8; pqcrypto_kyber::ffi::PQCLEAN_KYBER768_CLEAN_CRYPTO_PUBLICKEYBYTES],
        )?;
        let bytes = key.to_multi_bytes();
        let decoded_key = Idp2pAgreementPublicKey::from_multi_bytes(&bytes)?;
        assert_eq!(key, decoded_key);
        Ok(())
    }
}
