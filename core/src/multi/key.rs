use std::io::Read;

use ed25519_dalek::{PublicKey, Signature, Verifier};
use serde::{de::Error as SerdeError, Deserialize, Serialize};
use unsigned_varint::{encode as varint_encode, io::read_u64};

use crate::decode_base;

use super::{
    base::Idp2pBase, error::Idp2pMultiError, hash::Idp2pHash, key_digest::Idp2pKeyDigest,
    ED25519_CODE,
};

#[derive(PartialEq, Clone, Debug)]
pub enum Idp2pKey {
    Ed25519 { public: PublicKey },
}

impl Idp2pKey {
    pub fn new<T: AsRef<[u8]>>(typ: u64, bytes: T) -> Result<Self, Idp2pMultiError> {
        match typ {
            ED25519_CODE => Ok(Self::Ed25519 {
                public: PublicKey::from_bytes(bytes.as_ref())?,
            }),
            _ => Err(Idp2pMultiError::InvalidKeyCode),
        }
    }

    pub fn from_bytes<T: AsRef<[u8]>>(bytes: T) -> Result<Self, Idp2pMultiError> {
        let mut r = bytes.as_ref();
        let typ = read_u64(&mut r)?;
        let size = read_u64(&mut r)?;
        match typ {
            ED25519_CODE => {
                if size != 32 {
                    return Err(Idp2pMultiError::InvalidKeyCode);
                }
                let mut key_bytes = [0u8; 32];
                r.read_exact(&mut key_bytes)?;
                Ok(Self::Ed25519 {
                    public: PublicKey::from_bytes(key_bytes.as_ref())?,
                })
            }
            _ => Err(Idp2pMultiError::InvalidKeyCode),
        }
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        match &self {
            Self::Ed25519 { public } => {
                let mut type_buf = varint_encode::u64_buffer();
                let typ = varint_encode::u64(ED25519_CODE, &mut type_buf);
                let mut size_buf = varint_encode::u64_buffer();
                let size = varint_encode::u64(32, &mut size_buf);
                [typ, size, &public.to_bytes()].concat()
            }
        }
    }

    pub fn to_key_digest(&self) -> Idp2pKeyDigest {
        match self {
            Self::Ed25519 { public } => {
                let mh = Idp2pHash::default().digest(public.to_bytes());
                Idp2pKeyDigest::Ed25519 { multi_digest: mh }
            }
        }
    }

    pub fn to_id(&self) -> Vec<u8> {
        match self {
            Self::Ed25519 { public } => {
                let mh = Idp2pHash::default().digest(public.to_bytes());
                mh.to_bytes()
            }
        }
    }

    pub fn did_code(&self) -> String {
        match self {
            Self::Ed25519 { public: _ } => "Ed25519VerificationKey2020".to_string(),
        }
    }

    pub fn verify(&self, payload: &[u8], sig: &[u8]) -> Result<(), Idp2pMultiError> {
        match &self {
            Self::Ed25519 { public } => {
                let sig_bytes: [u8; 64] = sig.try_into()?;
                let signature = Signature::from(sig_bytes);
                let result = public.verify(payload, &signature)?;
                Ok(result)
            }
        }
    }
}

impl Serialize for Idp2pKey {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let s = Idp2pBase::default().encode(self.to_bytes());
        format!("{}", s).serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for Idp2pKey {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        let bytes = decode_base!(s)?;
        Ok(Self::from_bytes(bytes).map_err(SerdeError::custom)?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn enc_dec_test() -> Result<(), Idp2pMultiError> {
        let bytes = [0u8; 32];
        let key = Idp2pKey::new(ED25519_CODE, bytes)?;
        let key_bytes = key.to_bytes();
        let decoded_key = Idp2pKey::from_bytes(key_bytes)?;
        matches!(decoded_key, Idp2pKey::Ed25519 { public } if public.to_bytes() == bytes);
        Ok(())
    }
}
