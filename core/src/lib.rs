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
    InvalidDocumentDigest,
    #[error("Invalid next")]
    InvalidNext,
    #[error("Unknown")]
    Unknown,
}

pub type IdKeySecret = Vec<u8>;
pub type IdKey = Vec<u8>;
pub type IdKeyDigest = Vec<u8>;

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

pub fn hash(bytes: &[u8]) -> Vec<u8> {
    let mut hasher = Sha256::default();
    hasher.update(bytes);
    (&hasher.finalize()).to_vec()
}

pub fn encode(value: &[u8]) -> String {
    multibase::encode(Base::Base32Lower, value)
}

pub fn generate_cid<T: Sized + Serialize>(t: &T) -> String {
    let content = serde_json::to_string(t).unwrap();
    let hash = Code::Sha2_256.digest(content.as_bytes());
    let cid = Cid::new_v1(JSON_CODEC, hash);
    cid.to_string()
}

pub fn to_verification_keypair(secret: &[u8]) -> Keypair {
    let secret_key = SecretKey::from_bytes(secret).unwrap();
    let public_key: PublicKey = PublicKey::from(&secret_key);
    let mut public: Vec<u8> = public_key.to_bytes().to_vec();
    let mut new_secret = secret.to_vec();
    new_secret.append(&mut public);
    Keypair::from_bytes(&new_secret).unwrap()
}

pub fn create_secret_key() -> Vec<u8> {
    let mut key_data = [0u8; 32];
    let mut key_rng = thread_rng();
    key_rng.fill_bytes(&mut key_data);
    key_data.to_vec()
}

pub fn to_verification_publickey(secret_key: &[u8]) -> Vec<u8> {
    let keypair: Keypair = to_verification_keypair(secret_key);
    let public = keypair.public.to_bytes().to_vec();
    public
}

pub fn to_key_agreement_publickey(secret: &[u8]) -> Vec<u8> {
    let secret_data: [u8; 32] = secret.try_into().unwrap();
    let secret_key = StaticSecret::from(secret_data);
    let public_key = x25519_dalek::PublicKey::from(&secret_key);
    let public: Vec<u8> = public_key.to_bytes().to_vec();
    public
}

pub fn to_shared_secret(secret: &[u8], public: &[u8]) -> x25519_dalek::SharedSecret{
    let secret_data: [u8; 32] = secret.try_into().unwrap();
    let public_data: [u8; 32] = public.try_into().unwrap();
    let sender_secret = StaticSecret::from(secret_data);
    let receiver_public = x25519_dalek::PublicKey::from(public_data);
    sender_secret.diffie_hellman(&receiver_public)
}

macro_rules! check {
    ($e: expr, $err: expr) => {{
        if !$e {
            return Err($err);
        }
    }};
}

pub mod did;
pub mod did_doc;
pub mod eventlog;
pub mod microledger;
pub mod did_comm;

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

    #[test]
    fn proof_test() {
        let secret = "bclc5pn2tfuhkqmupbr3lkyc5o4g4je6glfwkix6nrtf7hch7b3kq";
        let keypair = to_verification_keypair(&multibase::decode(secret).unwrap().1);
        let sig = keypair.sign(&vec![0]);
        let public_key_bytes = to_verification_publickey(&multibase::decode(secret).unwrap().1);
        let public_key = PublicKey::from_bytes(&public_key_bytes).unwrap();
        let is_valid = public_key.verify(&vec![0], &sig).is_ok();
        assert!(is_valid);
    }

    #[test]
    fn to_verification_key_test(){
       let secret = "bclc5pn2tfuhkqmupbr3lkyc5o4g4je6glfwkix6nrtf7hch7b3kq";
       let expected = "brgzkmbdnyevdth3sczvxjumd6bdl6ngn6eqbsbpazuvq42bfzk2a";
       let public_key = to_verification_publickey(&multibase::decode(secret).unwrap().1);
       assert_eq!(encode(&public_key), expected);
    }

    #[test]
    fn to_agreement_key_test(){
        let secret = "bclc5pn2tfuhkqmupbr3lkyc5o4g4je6glfwkix6nrtf7hch7b3kq";
        let expected = "bbgitzmdocc3y2gvcmtiihr2gyw4xjppux7ea3gdo6afwy6gbrmpa";
        let public_key = to_key_agreement_publickey(&multibase::decode(secret).unwrap().1);
        assert_eq!(encode(&public_key), expected);
    }

    #[test]
    fn to_shared_key_test(){
        let alice_secret = create_secret_key();
        let bob_secret = create_secret_key();
        let alice_public = to_key_agreement_publickey(&alice_secret);
        let bob_public = to_key_agreement_publickey(&bob_secret);
        let alice_shared = to_shared_secret(&alice_secret, &bob_public);
        let bob_shared = to_shared_secret(&bob_secret, &alice_public);
        assert_eq!(alice_shared.as_bytes(), bob_shared.as_bytes());
    }
}
