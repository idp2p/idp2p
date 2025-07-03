use alloc::{string::String, vec::Vec};
use cid::multibase::{self, Base};
use sha2::{Digest, Sha256};

use crate::error::CommonError;

pub fn encode<T: AsRef<[u8]>>(input: T) -> String {
    multibase::encode(Base::Base32Lower, input)
}

pub fn decode(input: &str) -> Result<Vec<u8>, CommonError> {
    Ok(multibase::decode(input)?.1)
}

pub fn sha256_hash(content: &[u8]) -> [u8; 32] {
    let digest: [u8; 32] = Sha256::digest(content)
        .try_into().expect("Digest must be 32 bytes");
    digest
}