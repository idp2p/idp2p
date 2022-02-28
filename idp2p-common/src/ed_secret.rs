use anyhow::*;
use ed25519_dalek::{Keypair, PublicKey, SecretKey, Signer};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::convert::TryInto;
use x25519_dalek::StaticSecret;

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub struct EdSecret([u8; 32]);

impl EdSecret {
    pub fn new() -> Self {
        EdSecret(crate::create_random::<32>())
    }
    pub fn from(data: [u8; 32]) -> Self {
        EdSecret(data)
    }

    pub fn from_bytes(data: &[u8]) -> Self {
        EdSecret(data.try_into().unwrap())
    }

    pub fn from_str(s: &str) -> Result<Self> {
        Ok(EdSecret(crate::decode_sized(s)?))
    }

    pub fn to_bytes(&self) -> [u8; 32] {
        self.0
    }

    pub fn to_keypair(&self) -> Keypair {
        let secret_key = SecretKey::from_bytes(&self.0).unwrap();
        let public_key: PublicKey = PublicKey::from(&secret_key);
        let public: [u8; 32] = public_key.to_bytes();
        let mut new_secret: Vec<u8> = vec![];
        new_secret.extend(self.0);
        new_secret.extend(public);
        Keypair::from_bytes(&new_secret).unwrap()
    }

    pub fn to_publickey(&self) -> [u8; 32] {
        let keypair: Keypair = self.to_keypair();
        let public = keypair.public.to_bytes();
        public
    }

    pub fn to_publickey_digest(&self) -> Result<[u8; 32]> {
        let public = self.to_publickey();
        let hash = Sha256::digest(&public);
        let result: [u8; 32] = hash.try_into()?;
        Ok(result)
    }

    pub fn to_key_agreement(&self) -> [u8;32] {
        let secret_data: [u8; 32] = self.0.clone().try_into().unwrap();
        let secret_key = StaticSecret::from(secret_data);
        let public_key = x25519_dalek::PublicKey::from(&secret_key);
        public_key.to_bytes()
    }

    pub fn to_shared_secret(&self, public: [u8; 32]) -> x25519_dalek::SharedSecret {
        let sender_secret = StaticSecret::from(self.0);
        let receiver_public = x25519_dalek::PublicKey::from(public);
        sender_secret.diffie_hellman(&receiver_public)
    }

    pub fn sign<T: Serialize>(&self, t: &T) -> [u8; 64] {
        let payload_json = serde_json::to_string(t).unwrap();
        let bytes = payload_json.as_bytes();
        let keypair = self.to_keypair();
        keypair.sign(&bytes).to_bytes()
    }

    pub fn sign_str(&self, s: &str) -> [u8; 64] {
        let bytes = s.as_bytes();
        let keypair = self.to_keypair();
        keypair.sign(&bytes).to_bytes()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::encode;
    use ed25519_dalek::{PublicKey, Signer, Verifier};
    #[test]
    fn proof_test() {
        let secret_str = "bclc5pn2tfuhkqmupbr3lkyc5o4g4je6glfwkix6nrtf7hch7b3kq";
        let secret = EdSecret::from_str(secret_str).unwrap();
        let keypair = secret.to_keypair();
        let sig = keypair.sign(&vec![0]);
        let public_key_bytes = secret.to_publickey();
        let public_key = PublicKey::from_bytes(&public_key_bytes).unwrap();
        let is_valid = public_key.verify(&vec![0], &sig).is_ok();
        assert!(is_valid);
    }

    #[test]
    fn to_publickey_test() {
        let secret_str = "bclc5pn2tfuhkqmupbr3lkyc5o4g4je6glfwkix6nrtf7hch7b3kq";
        let secret = EdSecret::from_str(secret_str).unwrap();
        let expected = "brgzkmbdnyevdth3sczvxjumd6bdl6ngn6eqbsbpazuvq42bfzk2a";
        let public_key = secret.to_publickey();
        assert_eq!(encode(&public_key), expected);
    }

    #[test]
    fn to_agreement_key_test() {
        let secret_str = "bclc5pn2tfuhkqmupbr3lkyc5o4g4je6glfwkix6nrtf7hch7b3kq";
        let secret = EdSecret::from_str(secret_str).unwrap();
        let expected = "bbgitzmdocc3y2gvcmtiihr2gyw4xjppux7ea3gdo6afwy6gbrmpa";
        let public_key = secret.to_key_agreement();
        assert_eq!(encode(&public_key), expected);
    }

    #[test]
    fn to_shared_key_test() {
        let alice_secret = EdSecret::new();
        let bob_secret = EdSecret::new();
        let alice_public = alice_secret.to_key_agreement();
        let bob_public = bob_secret.to_key_agreement();
        let alice_shared = alice_secret.to_shared_secret(bob_public);
        let bob_shared = bob_secret.to_shared_secret(alice_public);
        assert_eq!(alice_shared.as_bytes(), bob_shared.as_bytes());
    }
}
