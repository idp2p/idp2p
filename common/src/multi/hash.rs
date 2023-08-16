use multihash::Multihash;

use super::error::Idp2pMultiError;

const SHA_256: u64 = 0x12;
const MAX_HASH_SIZE: usize = 64;

#[derive(PartialEq, Clone, Debug)]
pub struct Idp2pMultiHash(Multihash<MAX_HASH_SIZE>);

impl Idp2pMultiHash {
    pub fn new<T: AsRef<[u8]>>(content: T) -> Result<Self, Idp2pMultiError> {
        let mh = Multihash::<MAX_HASH_SIZE>::wrap(SHA_256, content.as_ref())?;
        Ok(Self(mh))
    }

    pub fn from_bytes(bytes: &[u8]) -> Result<Self, Idp2pMultiError> {
        Ok(Self(Multihash::from_bytes(bytes)?))
    }

    pub fn ensure<T: AsRef<[u8]>>(&self, content: T) -> Result<(), Idp2pMultiError> {
        let expected_mh = Multihash::<MAX_HASH_SIZE>::wrap(self.0.code(), content.as_ref())?;
        if self.0 != expected_mh {
            return Err(Idp2pMultiError::InvalidDigest);
        }
        Ok(())
    }

    pub fn to_bytes(&self) -> Vec<u8>{
        self.0.to_bytes()
    }
}
