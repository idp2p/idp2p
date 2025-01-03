use alloc::string::String;
use sha2::{Digest, Sha256};

pub fn to_hex_str<T: AsRef<[u8]>>(data: T) -> String{
    format!("0x{}", hex::encode(data))
}

pub fn sha256_hash(content: &[u8]) -> [u8; 32] {
    let digest: [u8; 32] = Sha256::digest(content)
        .try_into().expect("Digest must be 32 bytes");
    digest
}