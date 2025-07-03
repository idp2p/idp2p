use cid::Cid;
use cid::multihash::Multihash;

use crate::{error::CommonError, utils::sha256_hash, SHA2_256_CODE};

pub trait CidExt {
    fn ensure(&self, input: &[u8]) -> Result<(), CommonError>;
    fn create(code: u64, input: &[u8]) -> Result<Cid, CommonError>;
}

impl CidExt for Cid {
    fn ensure(&self, input: &[u8]) -> Result<(), CommonError> {
        match self.hash().code() {
            SHA2_256_CODE => {
                let input_digest = sha256_hash(input);
                if self.hash().digest() != input_digest.as_slice() {
                    return Err(CommonError::PayloadHashMismatch);
                }
            }
            _ => return Err(CommonError::UnsupportedHashAlgorithm(self.hash().code())),
        }
        Ok(())
    }

    fn create(code: u64, input: &[u8]) -> Result<Self, CommonError> {
        let input_digest = sha256_hash(input);
        let mh = Multihash::<64>::wrap(SHA2_256_CODE, &input_digest)?;
        Ok(Cid::new_v1(code, mh))
    }

}