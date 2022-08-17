use std::io::Read;

use ed25519_dalek::{PublicKey, Signature, Verifier};
use sha2::{Digest, Sha256};
use unsigned_varint::{encode as varint_encode, io::read_u64};

use super::{error::Idp2pMultiError, hasher::Idp2pHasher};

pub trait Verifier {
    
}
pub struct Idp2pPublicKey<const S: usize, V: Verifier>{
    public: [u8; S]
}


#[derive(PartialEq, Clone, Debug)]
pub enum Idp2pKeyCode {
    Ed25519 = 0xed,
    Dilithium2 = 0x1001,
    Winternitz = 0x1000,
}

impl TryInto<Idp2pKeyCode> for u64 {
    type Error = Idp2pMultiError;

    fn try_into(self) -> Result<Idp2pKeyCode, Self::Error> {
        todo!()
    }
}

impl Idp2pKeyCode {
    pub fn did_code(&self) -> String {
        match self {
            Self::Ed25519 => "Ed25519VerificationKey2020".to_string(),
            Self::Dilithium2 => todo!(),
            Self::Winternitz => todo!(),
        }
    }
}

#[derive(PartialEq, Clone, Debug)]
pub enum Idp2pKey {
    Ed25519 { public: [u8; 32] },
    Dilithium2 { public: [u8; 1312] },
    Winternitz { public: [u8; 32] },
}

impl Idp2pKey {
    pub fn new<T: AsRef<[u8]>>(code: Idp2pKeyCode, bytes: T) -> Result<Self, Idp2pMultiError> {
        match code {
            Idp2pKeyCode::Ed25519 => Ok(Self::Ed25519 {
                public: bytes.as_ref().try_into()?,
            }),
            _ => Err(Idp2pMultiError::InvalidKeyCode),
        }
    }

    pub fn from_bytes<T: AsRef<[u8]>>(bytes: T) -> Result<Self, Idp2pMultiError> {
        let mut r = bytes.as_ref();
        let typ: Idp2pKeyCode = read_u64(&mut r)?.try_into()?;
        let size = read_u64(&mut r)?;
        match typ {
            Idp2pKeyCode::Ed25519 => {
                let mut key_bytes = [0u8; 32];
                r.read_exact(&mut key_bytes)?;
                Ok(Self::Ed25519 { public: key_bytes })
            }
            _ => Err(Idp2pMultiError::InvalidKeyCode),
        }
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        match &self {
            Self::Ed25519 { public } => key_to_bytes(Idp2pKeyCode::Ed25519, 32, public),
            Self::Dilithium2 { public } => key_to_bytes(Idp2pKeyCode::Dilithium2, 32, public),
            Self::Winternitz { public } => key_to_bytes(Idp2pKeyCode::Dilithium2, 32, public),
        }
    }

    pub fn to_raw_bytes(&self) -> Vec<u8> {
        match &self {
            Self::Ed25519 { public } => public.to_vec(),
            Self::Dilithium2 { public } => public.to_vec(),
            Self::Winternitz { public } => public.to_vec(),
        }
    }

    pub fn to_key_digest(&self) -> Vec<u8> {
        match self {
            Self::Ed25519 { public } => {
                let mh = Idp2pHasher::default().digest(public);
                let mut type_buf = varint_encode::u64_buffer();
                let typ = varint_encode::u64(Idp2pKeyCode::Ed25519 as u64, &mut type_buf);
                [typ, &mh.to_bytes()].concat()
            }
            Self::Dilithium2 { public } => todo!(),
            Self::Winternitz { public } => todo!(),
        }
    }

    pub fn to_id(&self) -> Vec<u8> {
        match self {
            Self::Ed25519 { public } => generate_id(public),
            Self::Dilithium2 { public } => generate_id(public),
            Self::Winternitz { public } => generate_id(public),
        }
    }

    pub fn verify(&self, payload: &[u8], sig: &[u8]) -> Result<(), Idp2pMultiError> {
        match &self {
            Self::Ed25519 { public } => {
                let sig_bytes: [u8; 64] = sig.try_into()?;
                let signature = Signature::from(sig_bytes);
                let pk = PublicKey::from_bytes(public)?;
                let result = pk.verify(payload, &signature)?;
                Ok(result)
            }
            Self::Dilithium2 { public } => todo!(),
            Self::Winternitz { public } => todo!(),
        }
    }
}

fn generate_id(public: &[u8]) -> Vec<u8> {
    let h = Sha256::new().chain_update(public).finalize().to_vec();
    h[0..16].to_vec()
}

fn key_to_bytes(code: Idp2pKeyCode, size: u64, bytes: &[u8]) -> Vec<u8> {
    let mut type_buf = varint_encode::u64_buffer();
    let typ = varint_encode::u64(Idp2pKeyCode::Ed25519 as u64, &mut type_buf);
    let mut size_buf = varint_encode::u64_buffer();
    let size = varint_encode::u64(32, &mut size_buf);
    [typ, size, bytes].concat()
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn enc_dec_test() -> Result<(), Idp2pMultiError> {
        let bytes = [0u8; 32];
        let key = Idp2pKey::new(Idp2pKeyCode::Ed25519, bytes)?;
        let key_bytes = key.to_bytes();
        let decoded_key = Idp2pKey::from_bytes(key_bytes)?;
        matches!(decoded_key, Idp2pKey::Ed25519 { public } if public == bytes);
        Ok(())
    }
}
