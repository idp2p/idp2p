use cid::{
    multihash::{Code, MultihashDigest},
    Cid,
};
use ed25519_dalek::{Keypair, PublicKey, SecretKey};
use multibase::*;
use rand::prelude::*;
use serde::Serialize;
use sha2::{Digest, Sha256};
use std::convert::TryInto;
use x25519_dalek::StaticSecret;
const ED25519: &str = "Ed25519VerificationKey2020";
const X25519: &str = "X25519KeyAgreementKey2020";
const JSON_CODEC: u64 = 0x0200;

pub mod store;
pub mod secret;
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
        format!("{}", multibase::encode(Base::Base32Lower, value.as_ref())).serialize(serializer)
    }
}

pub fn encode(value: &[u8]) -> String {
    multibase::encode(Base::Base32Lower, value)
}

pub fn hash(bytes: &[u8]) -> Vec<u8> {
    let mut hasher = Sha256::default();
    hasher.update(bytes);
    (&hasher.finalize()).to_vec()
}

pub fn generate_cid<T: Sized + Serialize>(t: &T) -> String {
    let content = serde_json::to_string(t).unwrap();
    let hash = Code::Sha2_256.digest(content.as_bytes());
    let cid = Cid::new_v1(JSON_CODEC, hash);
    cid.to_string()
}

macro_rules! check {
    ($e: expr, $err: expr) => {{
        if !$e {
            return Err($err);
        }
    }};
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use ed25519_dalek::{PublicKey, Signature, Signer, Verifier};
    #[test]
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
        let cid = generate_cid(&data);
        assert_eq!(cid, expected);
    }
}