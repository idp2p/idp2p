use cid::Cid;
use multihash::Multihash;

use crate::{error::IdError, utils::sha256_hash, SHA2_256_CODE};

pub trait CidExt {
    fn ensure(&self, input: &[u8]) -> Result<(), IdError>;
    fn create(code: u64, input: &[u8]) -> Result<Cid, IdError>;
}

impl CidExt for Cid {
    fn ensure(&self, input: &[u8]) -> Result<(), IdError> {
        match self.hash().code() {
            SHA2_256_CODE => {
                let input_digest = sha256_hash(input)?;
                if self.hash().digest() != input_digest.as_slice() {
                    return Err(IdError::EnsureError {
                        expected: input_digest.to_vec(),
                        actual: self.hash().digest().to_vec(),
                    });
                }
            }
            _ => return Err(IdError::InvalidHashAlg(self.hash().code())),
        }
        Ok(())
    }

    fn create(code: u64, input: &[u8]) -> Result<Self, IdError> {
        let input_digest = sha256_hash(input)?;
        let mh =
            Multihash::<64>::wrap(SHA2_256_CODE, &input_digest).map_err(|_| IdError::Unknown)?;
        Ok(Cid::new_v1(code, mh))
    }
}
