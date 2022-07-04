use std::io::Read;

use rand::rngs::OsRng;
use serde::{de::Error as SerdeError, Deserialize, Serialize};
use sha2::{Digest, Sha256};
use unsigned_varint::{encode as varint_encode, io::read_u64};
use x25519_dalek::{EphemeralSecret, PublicKey};

use crate::decode_base;

use super::{base::Idp2pBase, error::Idp2pMultiError, X25519_CODE, agreement_secret::Idp2pAgreementSecret};

#[derive(PartialEq, Clone, Debug)]
pub enum Idp2pAgreementKey {
    X25519 { public: PublicKey },
}

impl Idp2pAgreementKey {
    pub fn new<T: AsRef<[u8]>>(typ: u64, bytes: T) -> Result<Self, Idp2pMultiError> {
        match typ {
            X25519_CODE => {
                let key_bytes: [u8; 32] = bytes.as_ref().try_into()?;
                Ok(Self::X25519 {
                    public: PublicKey::try_from(key_bytes)?,
                })
            }
            _ => Err(Idp2pMultiError::InvalidKeyCode),
        }
    }

    pub fn from_bytes<T: AsRef<[u8]>>(bytes: T) -> Result<Self, Idp2pMultiError> {
        let mut r = bytes.as_ref();
        let typ = read_u64(&mut r)?;
        let size = read_u64(&mut r)?;
        match typ {
            X25519_CODE => {
                if size != 32 {
                    return Err(Idp2pMultiError::InvalidKeyCode);
                }
                let mut key_bytes = [0u8; 32];
                r.read_exact(&mut key_bytes)?;
                Ok(Self::new(typ, key_bytes)?)
            }
            _ => Err(Idp2pMultiError::InvalidKeyCode),
        }
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        match &self {
            Self::X25519 { public } => {
                let mut type_buf = varint_encode::u64_buffer();
                let typ = varint_encode::u64(X25519_CODE, &mut type_buf);
                let mut size_buf = varint_encode::u64_buffer();
                let size = varint_encode::u64(32, &mut size_buf);
                [typ, size, &public.to_bytes()].concat()
            }
        }
    }

    pub fn create_shared_secret(&self) -> Result<(Vec<u8>, Vec<u8>), Idp2pMultiError> {
        match self {
            Idp2pAgreementKey::X25519 { public } => {
                let ephemeral_secret = EphemeralSecret::new(OsRng);
                let ephemeral_public = PublicKey::from(&ephemeral_secret);
                let shared_secret = ephemeral_secret.diffie_hellman(&public);
                Ok((
                    ephemeral_public.as_bytes().to_vec(),
                    shared_secret.to_bytes().to_vec(),
                ))
            }
        }
    }

    pub fn to_id(&self) -> Vec<u8> {
        match self {
            Self::X25519 { public } => {
                let h = Sha256::new()
                    .chain_update(public.to_bytes())
                    .finalize()
                    .to_vec();
                h[0..16].to_vec()
            }
        }
    }

    pub fn did_scheme(&self) -> String {
        match self {
            Self::X25519 { public: _ } => "X25519VerificationKey2020".to_string(),
        }
    }
}

impl Serialize for Idp2pAgreementKey {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let s = Idp2pBase::default().encode(self.to_bytes());
        format!("{}", s).serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for Idp2pAgreementKey {
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
        let key = Idp2pAgreementKey::new(X25519_CODE, bytes)?;
        let key_bytes = key.to_bytes();
        let decoded_key = Idp2pAgreementKey::from_bytes(key_bytes)?;
        matches!(decoded_key, Idp2pAgreementKey::X25519 { public } if public.to_bytes() == bytes);
        Ok(())
    }

    #[test]
    fn to_id_test() -> Result<(), Idp2pMultiError> {
        let bytes = [0u8; 32];
        let key = Idp2pAgreementKey::new(X25519_CODE, bytes)?;
        assert_eq!(key.to_id().len(), 16);
        Ok(())
    }
}
