use serde::Serialize;
use ed25519_dalek::{Keypair, SecretKey, PublicKey, Signer};
use rand::prelude::*;
use std::convert::TryInto;
use x25519_dalek::StaticSecret;
pub struct IdSecret(Vec<u8>);

impl IdSecret {
    pub fn new() -> IdSecret {
        let mut key_data = [0u8; 32];
        let mut key_rng = thread_rng();
        key_rng.fill_bytes(&mut key_data);
        IdSecret(key_data.to_vec())
    }

    pub fn from(data: &[u8]) -> IdSecret {
        IdSecret(data.to_vec())
    }

    pub fn from_str(s: &str) -> IdSecret {
        IdSecret(crate::decode(s).to_vec())
    }

    pub fn to_verification_keypair(&self) -> Keypair {
        let secret_key = SecretKey::from_bytes(&self.0).unwrap();
        let public_key: PublicKey = PublicKey::from(&secret_key);
        let mut public: Vec<u8> = public_key.to_bytes().to_vec();
        let mut new_secret = self.0.clone();
        new_secret.append(&mut public);
        Keypair::from_bytes(&new_secret).unwrap()
    }

    pub fn to_verification_publickey(&self) -> Vec<u8> {
        let keypair: Keypair = self.to_verification_keypair();
        let public = keypair.public.to_bytes().to_vec();
        public
    }

    pub fn to_key_agreement_publickey(&self) -> Vec<u8> {
        let secret_data: [u8; 32] = self.0.clone().try_into().unwrap();
        let secret_key = StaticSecret::from(secret_data);
        let public_key = x25519_dalek::PublicKey::from(&secret_key);
        let public: Vec<u8> = public_key.to_bytes().to_vec();
        public
    }

    pub fn to_shared_secret(&self, public: &[u8]) -> x25519_dalek::SharedSecret {
        let secret_data: [u8; 32] = self.0.clone().try_into().unwrap();
        let public_data: [u8; 32] = public.try_into().unwrap();
        let sender_secret = StaticSecret::from(secret_data);
        let receiver_public = x25519_dalek::PublicKey::from(public_data);
        sender_secret.diffie_hellman(&receiver_public)
    }

    pub fn sign<T: Serialize>(&self, t: &T) -> Vec<u8> {
        let payload_json = serde_json::to_string(t).unwrap();
        let bytes = payload_json.as_bytes();
        let keypair = self.to_verification_keypair();
        keypair.sign(&bytes).to_bytes().to_vec()
    }
}
