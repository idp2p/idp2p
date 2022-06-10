use crate::decode_base;

use super::{
    base::Idp2pBase, error::Idp2pMultiError, hash::Idp2pHash, key::Idp2pKey, ED25519_CODE,
};
use cid::multihash::Multihash;
use serde::{de::Error as SerdeError, Deserialize, Serialize};
use std::io::Read;
use unsigned_varint::{encode as varint_encode, io::read_u64};

#[derive(PartialEq, Clone, Debug)]
pub enum Idp2pKeyDigest {
    Ed25519 { multi_digest: Multihash },
}

impl Idp2pKeyDigest {
    pub fn from_bytes<T: AsRef<[u8]>>(bytes: T) -> Result<Self, Idp2pMultiError> {
        let mut r = bytes.as_ref();
        let typ = read_u64(&mut r)?;
        match typ {
            ED25519_CODE => {
                let mut key_bytes: Vec<u8> = vec![];
                r.read_to_end(&mut key_bytes)?;
                Ok(Self::Ed25519 {
                    multi_digest: Multihash::from_bytes(&key_bytes)?,
                })
            }
            _ => Err(Idp2pMultiError::InvalidKeyCode),
        }
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        match &self {
            Self::Ed25519 { multi_digest } => {
                let mut type_buf = varint_encode::u64_buffer();
                let typ = varint_encode::u64(ED25519_CODE, &mut type_buf);
                [typ, &multi_digest.to_bytes()].concat()
            }
        }
    }

    pub fn to_next_key(&self, public_bytes: &[u8]) -> Result<Idp2pKey, Idp2pMultiError> {
        match &self {
            Self::Ed25519 { multi_digest } => {
                let hasher = Idp2pHash::try_from(multi_digest.code())?;
                hasher.ensure(*multi_digest, public_bytes)?;
                Idp2pKey::new(ED25519_CODE, public_bytes)
            }
        }
    }
}

impl Serialize for Idp2pKeyDigest {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let s = Idp2pBase::default().encode(self.to_bytes());
        format!("{}", s).serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for Idp2pKeyDigest {
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
    use crate::multi::key::Idp2pKey;

    use super::*;
    #[test]
    fn test_slice() {
        let v: Vec<u8> = vec![1, 2, 3];
        let mut r = v.as_slice();
        let typ = read_u64(&mut r).unwrap();
        let mut other_bytes: Vec<u8> = vec![];
        r.read_to_end(&mut other_bytes).unwrap();
        eprintln!("{} {:?} {:?}", typ, other_bytes, v);
    }
    /*#[test]
    fn enc_dec_test() -> Result<(), Idp2pMultiError> {
        let bytes = [0u8; 32];
        let key = Idp2pKey::new(ED25519_CODE, bytes)?;
        let key_digest = key.to_key_digest();
        let digest_bytes = key_digest.to_bytes();
        let decoded_key = Idp2pKeyDigest::from_bytes(digest_bytes)?;
        matches!(decoded_key, Idp2pKeyDigest::Ed25519 { multi_digest } if multi_digest.code() == 0x12);
        Ok(())
    }*/
}
