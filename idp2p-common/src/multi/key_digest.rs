use crate::decode_base;

use super::{
    base::Idp2pBase,
    error::Idp2pMultiError,
    hasher::Idp2pHasher,
    key::{Idp2pKey, Idp2pKeyCode},
};
use cid::multihash::Multihash;
use serde::{de::Error as SerdeError, Deserialize, Serialize};
use std::io::Read;
use unsigned_varint::{encode as varint_encode, io::read_u64};

#[derive(PartialEq, Clone, Debug)]
pub struct Idp2pKeyDigest {
    code: Idp2pKeyCode,
    digest: Multihash,
}

impl Idp2pKeyDigest {
    pub fn from_bytes<T: AsRef<[u8]>>(bytes: T) -> Result<Self, Idp2pMultiError> {
        let mut r = bytes.as_ref();
        let typ: Idp2pKeyCode = read_u64(&mut r)?.try_into()?;
        let mut key_bytes: Vec<u8> = vec![];
        r.read_to_end(&mut key_bytes)?;
        Ok(Self {
            code: typ,
            digest: Multihash::from_bytes(&key_bytes)?,
        })
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        let mut type_buf = varint_encode::u64_buffer();
        let code = self.code.clone() as u64;
        let typ = varint_encode::u64(code, &mut type_buf);
        [typ, &self.digest.to_bytes()].concat()
    }

    pub fn to_next_key(&self, public_bytes: &[u8]) -> Result<Idp2pKey, Idp2pMultiError> {
        let hasher = Idp2pHasher::try_from(self.digest.code())?;
        hasher.ensure(self.digest, public_bytes)?;
        Idp2pKey::new(Idp2pKeyCode::Ed25519, public_bytes)
    }
}


#[cfg(test)]
mod tests {
    use crate::multi::key::Idp2pKey;

    use super::*;

    #[test]
    fn enc_dec_test() -> Result<(), Idp2pMultiError> {
        let bytes = [0u8; 32];
        let key = Idp2pKey::new(Idp2pKeyCode::Ed25519, bytes)?;
        let key_digest = key.to_key_digest();
        let decoded_key = Idp2pKeyDigest::from_bytes(key_digest)?;
        assert_eq!(decoded_key.digest.code(), 0x12);
        Ok(())
    }
}
