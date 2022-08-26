use ed25519_dalek::{SecretKey, PublicKey, Keypair};

use crate::multi::error::Idp2pMultiError;

use super::{VerificationPublicKey, VerificationKeypair};

#[derive(PartialEq, Clone, Debug)]
pub struct Ed25519Keypair {
    secret: [u8; 32],
    public: [u8; 32],
}

#[derive(PartialEq, Clone, Debug)]
pub struct Ed25519PublicKey([u8;32]);

impl VerificationPublicKey for Ed25519PublicKey {
    fn pub_bytes(&self) -> Vec<u8> {
        self.0.to_vec()
    }

    fn verify(&self, payload: &[u8], sig: &[u8]) -> Result<bool, Idp2pMultiError> {
        todo!()
    }
}

impl VerificationKeypair for Ed25519Keypair{
    type PublicKeyType = Ed25519PublicKey;

    fn priv_bytes(&self) -> Vec<u8> {
        todo!()
    }

    fn to_public_key(&self) -> Self::PublicKeyType {
        todo!()
    }
}

impl Ed25519Keypair{
    pub fn from_secret(secret: [u8;32]) -> Result<Self, Idp2pMultiError>{
        let sk = SecretKey::from_bytes(&secret)?;
        let pk: PublicKey = (&sk).into();
        let keypair = Ed25519Keypair {
            public: *pk.as_bytes(),
            secret: secret,
        };
        Ok(keypair)
    }

    fn to_ed_keypair(&self) -> Result<Keypair, Idp2pMultiError>{
        let sk = SecretKey::from_bytes(&self.secret)?;
        let pk: PublicKey = PublicKey::from_bytes(&self.public)?;
        let keypair = Keypair {
            public: pk,
            secret: sk,
        };
        Ok(keypair)
    }
}