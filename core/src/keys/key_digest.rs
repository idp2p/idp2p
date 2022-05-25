use std::io::Read;

use super::{
    ED25519_CODE, error::MultiKeyError,
};
use cid::multihash::Multihash;
use unsigned_varint::{encode as varint_encode, io::read_u64};

#[derive(PartialEq, Clone)]
pub enum Idp2pKeyDigest {
    Ed25519 { multi_digest: Multihash },
}

impl Idp2pKeyDigest {
    pub fn from_bytes<T: AsRef<[u8]>>(bytes: T) -> Result<Self, MultiKeyError> {
        let mut r = bytes.as_ref();
        let typ = read_u64(r)?;
        match typ {
            ED25519_CODE => {
                let mut key_bytes: Vec<u8> = vec![];
                r.read_to_end(&mut key_bytes)?;
                Ok(Self::Ed25519 {
                    multi_digest: Multihash::from_bytes(&key_bytes)?,
                })
            }
            _ => Err(MultiKeyError::InvalidKeyCode),
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
}