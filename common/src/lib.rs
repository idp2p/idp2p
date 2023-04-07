pub mod multi;
use serde::Serialize;
use sha2::Digest;

pub fn digest<T: Serialize>(payload: &T) -> Result<String, std::io::Error> {
    Ok(digest_str(&serde_json::to_string(payload)?))
}

pub fn digest_str(payload: &str) -> String {
    to_hex_str(sha2::Sha256::digest(payload.as_bytes()))
}

pub fn to_hex_str<T: AsRef<[u8]>>(data: T) -> String{
   format!("0x{}", hex::encode(data))
}

pub fn create_random<const N: usize>() -> [u8; N] {
    let mut key_data = [0u8; N];
    let mut key_rng = thread_rng();
    key_rng.fill_bytes(&mut key_data);
    key_data
}

