use crate::{error::Idp2pMultiError, random::create_random};

use super::{ AgreementPublicBehaviour, AgreementShared, AgreementSecretBehaviour};

use rand::rngs::OsRng;
use x25519_dalek::{EphemeralSecret, PublicKey, StaticSecret};

pub const X25519_PUBLIC_SIZE: usize = 32;
pub const X25519_SECRET_SIZE: usize = 32;

#[derive(PartialEq, Clone, Debug)]
pub struct X25519PublicKey(pub(super) [u8; X25519_PUBLIC_SIZE]);

#[derive(PartialEq, Clone, Debug)]
pub struct X25519Keypair {
    secret: [u8; X25519_SECRET_SIZE],
    public: [u8; X25519_PUBLIC_SIZE],
}

impl AgreementSecretBehaviour for X25519Keypair {
    type PublicKeyType = X25519PublicKey;
    fn priv_as_bytes<'a>(&'a self) -> &'a [u8] {
        &self.secret
    }

    fn to_public_key(&self) -> X25519PublicKey {
        X25519PublicKey(self.public)
    }

    fn resolve_shared_secret(&self, data: &[u8]) -> Result<Vec<u8>, Idp2pMultiError> {
        let sender_secret = StaticSecret::from(self.secret);
        let public_bytes: [u8; X25519_PUBLIC_SIZE] = data.try_into()?;
        let pk = PublicKey::try_from(public_bytes)?;
        let shared_secret = sender_secret.diffie_hellman(&pk);
        Ok(shared_secret.to_bytes().to_vec())
    }
}

impl X25519Keypair {
    pub fn generate() -> Self{
        let secret = create_random::<X25519_SECRET_SIZE>();
        Self::from_secret_bytes(secret)
    }

    pub fn from_secret_bytes(secret: [u8; X25519_SECRET_SIZE]) -> Self {
        let static_secret = StaticSecret::from(secret);
        let public_key = PublicKey::from(&static_secret);
        Self {
            public: *public_key.as_bytes(),
            secret: secret,
        }
    }
}


impl From<[u8; X25519_PUBLIC_SIZE]> for X25519PublicKey{
    fn from(value: [u8; X25519_PUBLIC_SIZE]) -> Self {
        Self(value)
    }
}

impl TryFrom<&[u8]> for X25519PublicKey{
    type Error = Idp2pMultiError;

    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        let public: [u8; X25519_PUBLIC_SIZE] = value.try_into()?;
        Ok(public.into())
    }
}

impl AgreementPublicBehaviour for X25519PublicKey {
    fn as_bytes<'a>(&'a self) -> &'a [u8] {
        &self.0
    }

    fn create_shared(&self) -> Result<AgreementShared, Idp2pMultiError> {
        let ephemeral_secret = EphemeralSecret::new(OsRng);
        let ephemeral_public = PublicKey::from(&ephemeral_secret);
        let pk = PublicKey::try_from(self.0)?;
        let shared_secret = ephemeral_secret.diffie_hellman(&pk);
        Ok(AgreementShared {
            secret: shared_secret.as_bytes().to_vec(),
            data: ephemeral_public.to_bytes().to_vec(),
        })
    }
}
