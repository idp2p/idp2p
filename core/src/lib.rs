use cid::{
    multihash::{Code, MultihashDigest},
    Cid,
};
use ed25519_dalek::{Keypair, PublicKey, SecretKey};
use multibase::*;
use rand::prelude::*;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use thiserror::Error;
use x25519_dalek::StaticSecret;
const ED25519: &str = "Ed25519VerificationKey2020";
const X25519: &str = "X25519KeyAgreementKey2020";
const JSON_CODEC: u64 = 0x0200;

#[derive(Error, Debug)]
pub enum IdentityError {
    #[error("Invalid id")]
    InvalidId,
    #[error("Invalid ledger")]
    InvalidLedger,
    #[error("Invalid previous")]
    InvalidPrevious,
    #[error("Invalid event signature")]
    InvalidEventSignature,
    #[error("Invalid signer")]
    InvalidSigner,
    #[error("Invalid recovery")]
    InvalidRecovery,
    #[error("Invalid doc digest")]
    InvalidDocumentDigest,
    #[error("Invalid next")]
    InvalidNext,
    #[error("Unknown")]
    Unknown,
}

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub struct SignerKey {
    #[serde(rename = "type")]
    pub typ: String,
    #[serde(with = "encode_me")]
    pub public: Vec<u8>,
}

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub struct RecoveryKey {
    #[serde(rename = "type")]
    pub typ: String,
    #[serde(with = "encode_me")]
    pub digest: Vec<u8>,
}

impl SignerKey {
    pub fn new(public: Vec<u8>) -> SignerKey {
        SignerKey {
            typ: ED25519.to_string(),
            public: public,
        }
    }
}

impl RecoveryKey {
    pub fn new(digest: Vec<u8>) -> RecoveryKey {
        RecoveryKey {
            typ: ED25519.to_string(),
            digest: digest,
        }
    }
}

pub mod encode_me {
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

pub fn hash(bytes: &[u8]) -> Vec<u8> {
    let mut hasher = Sha256::default();
    hasher.update(bytes);
    (&hasher.finalize()).to_vec()
}

pub fn encode(value: Vec<u8>) -> String {
    multibase::encode(Base::Base32Lower, &value)
}

pub fn generate_cid<T: Sized + Serialize>(t: &T) -> String {
    let content = serde_json::to_string(t).unwrap();
    let hash = Code::Sha2_256.digest(content.as_bytes());
    let cid = Cid::new_v1(JSON_CODEC, hash);
    cid.to_string()
}

pub fn to_keypair(secret: Vec<u8>) -> Keypair {
    let mut secret = secret.clone();
    let secret_key = SecretKey::from_bytes(&secret).unwrap();
    let public_key: PublicKey = PublicKey::from(&secret_key);
    let mut public: Vec<u8> = public_key.to_bytes().to_vec();
    secret.append(&mut public);
    Keypair::from_bytes(&secret).unwrap()
}

pub fn create_verification_key() -> (Vec<u8>, Vec<u8>) {
    let mut key_data = [0u8; 32];
    let mut key_rng = thread_rng();
    key_rng.fill_bytes(&mut key_data);
    let keypair: Keypair = to_keypair(key_data.to_vec());
    let public = keypair.public.to_bytes().to_vec();
    let secret = keypair.secret.as_bytes().to_vec();
    (secret, public)
}

pub fn to_verification_publickey(secret_key: Vec<u8>) -> Vec<u8>{
    let keypair: Keypair = to_keypair(secret_key);
    let public = keypair.public.to_bytes().to_vec();
    public
}

pub fn create_key_agreement() -> (Vec<u8>, Vec<u8>) {
    let mut key_data = [0u8; 32];
    let mut key_rng = thread_rng();
    key_rng.fill_bytes(&mut key_data);
    let secret_key = StaticSecret::from(key_data);
    let public_key = x25519_dalek::PublicKey::from(&secret_key);
    let public: Vec<u8> = public_key.to_bytes().to_vec();
    (key_data.to_vec(), public)
}

macro_rules! check {
    ($e: expr, $err: expr) => {{
        if !$e {
            return Err($err);
        }
    }};
}

pub mod did_doc;
pub mod eventlog;
pub mod microledger;
pub mod did;

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn hash_test() {
        let v = vec![0,1];
        let digest = hash(&v);
        let hex = multibase::encode(Base::Base16Lower, digest);
        assert_eq!(hex, "fb413f47d13ee2fe6c845b2ee141af81de858df4ec549a58b7970bb96645bc8d2");
    }
}

