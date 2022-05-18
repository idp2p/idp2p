pub mod base64url;
pub mod bip32;
pub mod cipher;

use rand::prelude::*;
pub use anyhow;
pub use chrono;
pub use ed25519_dalek;
pub use log;
pub use multibase;
pub use rand;
pub use regex;
pub use serde_json;
pub use serde_with;
pub use sha2;
pub use thiserror;
pub use cid;

pub fn create_random<const N: usize>() -> [u8; N] {
    let mut key_data = [0u8; N];
    let mut key_rng = thread_rng();
    key_rng.fill_bytes(&mut key_data);
    key_data
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    /*#[test]
    fn hash_test() {
        let data = json!({
            "name": "John Doe"
        });
        let expected = "botmu6ay364t223hj4akn7amds6rpwquuavx54demvy5e4vkn5uuq";
        let digest = hash(serde_json::to_string(&data).unwrap().as_bytes());
        let result = encode(&digest);
        assert_eq!(result, expected);
    }
    #[test]
    fn cid_test() {
        let data = json!({
            "name": "John Doe"
        });
        let expected = "bagaaieraotmu6ay364t223hj4akn7amds6rpwquuavx54demvy5e4vkn5uuq";
        let cid = generate_json_cid(&data).unwrap();
        assert_eq!(cid, expected);
    }

    #[test]
    fn multihash_test() {
        let digest = Code::Sha2_256.digest(b"hello world!");
        eprintln!("{:?}", digest.digest());
    }*/
}
