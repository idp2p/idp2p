use ed25519_dalek::{
    Keypair, PublicKey, SecretKey, Signature, Signer as EdSigner, Verifier as EdVerifier,
};

use crate::{multi::error::Idp2pMultiError, random::create_random};

use super::{Signer, Verifier};
pub const ED25519_PUBLIC_SIZE: usize = 32;
pub const ED25519_SECRET_SIZE: usize = 32;
pub const ED25519_SIG_SIZE: usize = 64;

#[derive(PartialEq, Clone, Debug)]
pub struct Ed25519Keypair {
    secret: [u8; ED25519_PUBLIC_SIZE],
    public: [u8; ED25519_SECRET_SIZE],
}

#[derive(PartialEq, Clone, Debug)]
pub struct Ed25519PublicKey([u8; ED25519_PUBLIC_SIZE]);

impl Verifier for Ed25519PublicKey {
    fn pub_bytes(&self) -> Vec<u8> {
        self.0.to_vec()
    }

    fn verify(&self, payload: &[u8], sig: &[u8]) -> Result<bool, Idp2pMultiError> {
        let pk = PublicKey::from_bytes(&self.0)?;
        let sig_bytes: [u8; ED25519_SIG_SIZE] = sig.try_into()?;
        let signature = Signature::from(sig_bytes);
        pk.verify(payload, &signature)?;
        Ok(true)
    }
}

impl Signer for Ed25519Keypair {
    type PublicKeyType = Ed25519PublicKey;

    fn priv_bytes(&self) -> Vec<u8> {
        self.secret.to_vec()
    }

    fn to_public_key(&self) -> Self::PublicKeyType {
        Ed25519PublicKey(self.public)
    }

    fn sign(&self, payload: &[u8]) -> Result<Vec<u8>, Idp2pMultiError> {
        let keypair = self.to_ed_keypair()?;
        let sig: [u8; ED25519_SIG_SIZE] = keypair.sign(payload).to_bytes();
        Ok(sig.to_vec())
    }
}

impl Ed25519Keypair {
    pub fn generate() -> Self{
        let secret = create_random::<ED25519_SECRET_SIZE>();
        Self::from_secret(secret)
    }

    pub fn from_secret(secret: [u8; ED25519_PUBLIC_SIZE]) -> Self {
        let sk = SecretKey::from_bytes(&secret).unwrap();
        let pk: PublicKey = (&sk).into();
        let keypair = Ed25519Keypair {
            public: *pk.as_bytes(),
            secret: secret,
        };
        keypair
    }

    fn to_ed_keypair(&self) -> Result<Keypair, Idp2pMultiError> {
        let sk = SecretKey::from_bytes(&self.secret)?;
        let pk: PublicKey = PublicKey::from_bytes(&self.public)?;
        let keypair = Keypair {
            public: pk,
            secret: sk,
        };
        Ok(keypair)
    }
}

impl From<[u8; ED25519_PUBLIC_SIZE]> for Ed25519PublicKey{
    fn from(bytes: [u8; ED25519_PUBLIC_SIZE]) -> Self {
        Self(bytes)
    }
}

impl TryFrom<&[u8]> for Ed25519PublicKey{
    type Error = Idp2pMultiError;

    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        let public: [u8; ED25519_PUBLIC_SIZE] = value.try_into()?;
        Ok(public.into())
    }
}
