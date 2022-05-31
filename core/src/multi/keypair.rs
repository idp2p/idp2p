use super::{agreement_key::Idp2pAgreementKey, error::Idp2pMultiError, key::Idp2pKey};
use ed25519_dalek::{Keypair, PublicKey, SecretKey, Signer};
use x25519_dalek::StaticSecret;

#[derive(Debug)]
pub enum Idp2pKeypair {
    Ed25519 { keypair: Keypair },
}

impl Idp2pKeypair {
    pub fn new_ed25519<T: AsRef<[u8]>>(secret_bytes: T) -> Result<Self, Idp2pMultiError> {
        let sk = SecretKey::from_bytes(secret_bytes.as_ref())?;
        let pk: PublicKey = (&sk).into();
        let sk = SecretKey::from_bytes(secret_bytes.as_ref())?;
        let keypair = Keypair {
            public: pk,
            secret: sk,
        };
        Ok(Self::Ed25519 { keypair: keypair })
    }

    pub fn sign<T: AsRef<[u8]>>(&self, payload: T) -> Vec<u8> {
        match self {
            Idp2pKeypair::Ed25519 { keypair } => keypair.sign(payload.as_ref()).to_bytes().to_vec(),
        }
    }

    pub fn to_key(&self) -> Idp2pKey{
        match self {
            Self::Ed25519 { keypair } => Idp2pKey::Ed25519 {
                public: keypair.public,
            },
        }
    }

    pub fn to_agreement_key(&self) -> Idp2pAgreementKey{
        match self {
            Self::Ed25519 { keypair } => {
                let secret_bytes = keypair.secret.to_bytes();
                let static_secret = StaticSecret::from(secret_bytes);
                let public_key = x25519_dalek::PublicKey::from(&static_secret);
                Idp2pAgreementKey::X25519 { public: public_key }
            }
        }
    }

    pub fn to_shared_secret(&self, public: [u8; 32]) -> x25519_dalek::SharedSecret {
        match self {
            Idp2pKeypair::Ed25519 { keypair } =>{
                let secret_bytes = keypair.secret.to_bytes();
                let sender_secret = StaticSecret::from(secret_bytes);
                let receiver_public = x25519_dalek::PublicKey::from(public);
                sender_secret.diffie_hellman(&receiver_public)
            }
        }
    }
}
