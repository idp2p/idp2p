use crate::multi::error::Idp2pMultiError;

use super::{AgreementKeypair, AgreementPublicKey, AgreementShared};

use rand::rngs::OsRng;
use x25519_dalek::{EphemeralSecret, PublicKey, StaticSecret};

#[derive(PartialEq, Clone, Debug)]
pub struct X25519PublicKey(pub(super) [u8; 32]);

#[derive(PartialEq, Clone, Debug)]
pub struct X25519Keypair {
    secret: [u8; 32],
    public: [u8; 32],
}

impl AgreementKeypair for X25519Keypair {
    type PublicKeyType = X25519PublicKey;
    fn priv_bytes(&self) -> Vec<u8> {
        self.secret.to_vec()
    }

    fn to_public_key(&self) -> X25519PublicKey {
        X25519PublicKey(self.public)
    }

    fn resolve_shared_secret(&self, data: &[u8]) -> Result<Vec<u8>, Idp2pMultiError> {
        let sender_secret = StaticSecret::from(self.secret);
        let public_bytes: [u8; 32] = data.try_into()?;
        let pk = PublicKey::try_from(public_bytes)?;
        let shared_secret = sender_secret.diffie_hellman(&pk);
        Ok(shared_secret.to_bytes().to_vec())
    }
}

impl X25519Keypair {
    pub fn from_secret_bytes(secret: [u8; 32]) -> Self {
        let static_secret = StaticSecret::from(secret);
        let public_key = PublicKey::from(&static_secret);
        Self {
            public: *public_key.as_bytes(),
            secret: secret,
        }
    }
}

impl X25519PublicKey {
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, Idp2pMultiError> {
        Ok(Self(bytes.try_into()?))
    }
}

impl AgreementPublicKey for X25519PublicKey {
    fn pub_bytes(&self) -> Vec<u8> {
        self.0.to_vec()
    }

    fn create_data(&self) -> Result<AgreementShared, Idp2pMultiError> {
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
