use sha2::{Digest, Sha256};

use crate::error::CommonError;

pub fn to_hex_str<T: AsRef<[u8]>>(data: T) -> String{
    format!("0x{}", hex::encode(data))
}

pub fn sha256_hash(content: &[u8]) -> Result<[u8; 32], CommonError> {
    let digest: [u8; 32] = Sha256::digest(content)
        .try_into()
        .map_err(|_| CommonError::Unknown)?;
    Ok(digest)
}