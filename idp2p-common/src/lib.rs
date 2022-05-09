use chacha20poly1305::aead::{Aead, NewAead};
use chacha20poly1305::{ChaCha20Poly1305, Key, Nonce};
use cid::multihash::Multihash;
use cid::{
    multihash::{Code, MultihashDigest},
    Cid,
};
use multibase::*;
use pbkdf2::{
    password_hash::{Error, PasswordHasher, SaltString},
    Pbkdf2,
};
use rand::prelude::*;
use serde::Serialize;
use sha2::{Digest, Sha256};

pub const IDP2P_ED25519: &str = "Idp2pEd25519Key";
pub const ED25519: &str = "Ed25519VerificationKey2020";
pub const X25519: &str = "X25519KeyAgreementKey2020";
pub type IdKeySecret = Vec<u8>;
pub type IdKey = Vec<u8>;
pub type IdKeyDigest = Vec<u8>;
pub mod agreement_key;
pub mod base64url;
pub mod bip32;
pub mod hash;
pub mod id_proof;
pub mod key;
pub mod key_digest;
pub mod secret;
pub use anyhow;
pub use chrono;
pub use cid::multihash;
pub use ed25519_dalek;
pub use log;
pub use multibase;
pub use rand;
pub use regex;
pub use serde_json;
pub use serde_with;
pub use sha2;
pub use thiserror;

pub mod encode_vec {
    use multibase::Base;
    use serde::{Deserialize, Deserializer, Serialize, Serializer};

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Vec<u8>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        let (_, data) = multibase::decode(&s).unwrap();
        Ok(data)
    }

    pub fn serialize<S, T>(value: &T, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
        T: AsRef<[u8]>,
    {
        format!("{}", multibase::encode(Base::Base64Url, value.as_ref())).serialize(serializer)
    }
}

pub fn encode(value: &[u8]) -> String {
    multibase::encode(Base::Base32Lower, value)
}

pub fn decode(s: &str) -> Vec<u8> {
    multibase::decode(s).unwrap().1
}

pub fn decode_sized<const N: usize>(s: &str) -> anyhow::Result<[u8; N]> {
    let r = multibase::decode(s)?.1;
    let data: [u8; N] = r.try_into().expect("Data size is not equal to given size");
    Ok(data)
}

pub fn create_random<const N: usize>() -> [u8; N] {
    let mut key_data = [0u8; N];
    let mut key_rng = thread_rng();
    key_rng.fill_bytes(&mut key_data);
    key_data
}

pub fn get_enc_key(password: &str, salt: &[u8]) -> anyhow::Result<Vec<u8>, Error> {
    let salt_b64 = crate::multibase::encode(crate::multibase::Base::Base64, salt);
    let salt = SaltString::new(&salt_b64[1..])?;
    let enc_key = Pbkdf2.hash_password(password.as_bytes(), &salt)?;
    let enc_key_hash = enc_key.hash.unwrap();
    Ok(enc_key_hash.as_bytes().to_vec())
}

pub fn encrypt(enc_key_bytes: &[u8], iv: &[u8], content: &[u8]) -> Result<Vec<u8>> {
    let enc_key = Key::from_slice(&enc_key_bytes);
    let cipher = ChaCha20Poly1305::new(enc_key);
    let nonce = Nonce::from_slice(iv);
    let ciphertext = cipher.encrypt(nonce, content).expect("encryption failure!");
    Ok(ciphertext)
}

pub fn decrypt(enc_key_bytes: &[u8], iv: &[u8], ciphertext: &[u8]) -> Result<Vec<u8>> {
    let enc_key = Key::from_slice(&enc_key_bytes);
    let cipher = ChaCha20Poly1305::new(enc_key);
    let nonce = Nonce::from_slice(iv);
    let result = cipher.decrypt(nonce, ciphertext).unwrap();
    Ok(result)
}

pub fn is_idp2p(id: &str) -> bool {
    let re = regex::Regex::new(r"did:p2p:*").unwrap();
    re.is_match(id)
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
    }*/

    #[test]
    fn multihash_test() {
        let digest = Code::Sha2_256.digest(b"hello world!");
        eprintln!("{:?}", digest.digest());
    }
}
