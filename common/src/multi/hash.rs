use cid::multihash::{Multihash, MultihashDigest};

use super::error::Idp2pMultiError;
#[derive(Debug, Clone)]
pub enum Idp2pHash {
    Sha256 = 0x12,
}

impl TryFrom<u64> for Idp2pHash {
    type Error = Idp2pMultiError;

    fn try_from(value: u64) -> Result<Self, Self::Error> {
        match value {
            0x12 => Ok(Idp2pHash::Sha256),
            _ => Err(Idp2pMultiError::HashAlgError),
        }
    }
}

impl Default for Idp2pHash {
    fn default() -> Self {
        Idp2pHash::Sha256
    }
}

impl Idp2pHash {
    pub fn from_bytes(bytes: &[u8])-> Result<Multihash, Idp2pMultiError>{
        Ok(Multihash::from_bytes(bytes)?)
    }
    pub fn digest<T: AsRef<[u8]>>(&self, content: T) -> Multihash {
        match self {
            Idp2pHash::Sha256 => cid::multihash::Code::Sha2_256.digest(content.as_ref()),
        }
    }

    pub fn ensure(&self, digest: Multihash, payload: &[u8]) -> Result<(), Idp2pMultiError>{
        let mh = self.digest(payload);
        if mh != digest{
           return Err(Idp2pMultiError::InvalidDigest)
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn hash_test() {
        let mh = Idp2pHash::default().digest(vec![]);
        assert_eq!(mh.code(), 0x12);
        assert_eq!(mh.size(), 32);
    }
}
