use crate::hash;
use ed25519_dalek::{Keypair, PublicKey, SecretKey, Signer};
use serde::{Serialize, Deserialize};
use std::convert::TryInto;
use x25519_dalek::StaticSecret;

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub struct IdSecret(Vec<u8>);

impl IdSecret {
    pub fn new() -> Self {
        IdSecret(crate::create_random::<32>().to_vec())
    }

    pub fn from(data: &[u8]) -> Self {
        IdSecret(data.to_vec())
    }

    pub fn from_str(s: &str) -> Self {
        IdSecret(crate::decode(s).to_vec())
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        self.0.clone()
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

    pub fn to_publickey_digest(&self) -> Vec<u8> {
        hash(&self.to_verification_publickey())
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::encode;
    use ed25519_dalek::{PublicKey, Signer, Verifier};
    #[test]
    fn proof_test() {
        let secret = IdSecret::from_str("bclc5pn2tfuhkqmupbr3lkyc5o4g4je6glfwkix6nrtf7hch7b3kq");
        let keypair = secret.to_verification_keypair();
        let sig = keypair.sign(&vec![0]);
        let public_key_bytes = secret.to_verification_publickey();
        let public_key = PublicKey::from_bytes(&public_key_bytes).unwrap();
        let is_valid = public_key.verify(&vec![0], &sig).is_ok();
        assert!(is_valid);
    }

    #[test]
    fn to_verification_key_test() {
        let secret = IdSecret::from_str("bclc5pn2tfuhkqmupbr3lkyc5o4g4je6glfwkix6nrtf7hch7b3kq");
        let expected = "brgzkmbdnyevdth3sczvxjumd6bdl6ngn6eqbsbpazuvq42bfzk2a";
        let public_key = secret.to_verification_publickey();
        assert_eq!(encode(&public_key), expected);
    }

    #[test]
    fn to_agreement_key_test() {
        let secret = IdSecret::from_str("bclc5pn2tfuhkqmupbr3lkyc5o4g4je6glfwkix6nrtf7hch7b3kq");
        let expected = "bbgitzmdocc3y2gvcmtiihr2gyw4xjppux7ea3gdo6afwy6gbrmpa";
        let public_key = secret.to_key_agreement_publickey();
        assert_eq!(encode(&public_key), expected);
    }

    #[test]
    fn to_shared_key_test() {
        let alice_secret = IdSecret::new();
        let bob_secret = IdSecret::new();
        let alice_public = alice_secret.to_key_agreement_publickey();
        let bob_public = bob_secret.to_key_agreement_publickey();
        let alice_shared = alice_secret.to_shared_secret(&bob_public);
        let bob_shared = bob_secret.to_shared_secret(&alice_public);
        assert_eq!(alice_shared.as_bytes(), bob_shared.as_bytes());
    }
}
