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

    pub fn from_str(s: &str) -> Result<Self> {
        Ok(EdSecret(crate::decode_(s)?))
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
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::encode;
    use ed25519_dalek::{PublicKey, Signer, Verifier};
}
