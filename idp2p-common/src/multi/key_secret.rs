use super::{error::Idp2pMultiError, key::Idp2pKey};
use ed25519_dalek::{Keypair, PublicKey, SecretKey, Signer};

#[derive(Debug, Clone, PartialEq)]
pub enum Idp2pKeySecret {
    Ed25519 { secret: [u8; 32] },
}

impl Idp2pKeySecret {
    pub fn from_bytes<T: AsRef<[u8]>>(secret_bytes: T) -> Result<Self, Idp2pMultiError> {
        Ok(Self::Ed25519 {
            secret: secret_bytes.as_ref().try_into()?,
        })
    }

    pub fn sign<T: AsRef<[u8]>>(&self, payload: T) -> Result<Vec<u8>, Idp2pMultiError> {
        match self {
            Idp2pKeySecret::Ed25519 { secret } => {
                let keypair = to_ed_keypair(secret)?;
                Ok(keypair.sign(payload.as_ref()).to_bytes().to_vec())
            }
        }
    }

    pub fn to_key(&self) -> Result<Idp2pKey, Idp2pMultiError> {
        match self {
            Self::Ed25519 { secret } => {
                let keypair = to_ed_keypair(secret)?;
                Ok(Idp2pKey::Ed25519 {
                    public: keypair.public,
                })
            }
        }
    }
}

fn to_ed_keypair(secret: &[u8]) -> Result<Keypair, Idp2pMultiError> {
    let sk = SecretKey::from_bytes(secret)?;
    let pk: PublicKey = (&sk).into();
    let sk = SecretKey::from_bytes(secret)?;
    let keypair = Keypair {
        public: pk,
        secret: sk,
    };
    Ok(keypair)
}
