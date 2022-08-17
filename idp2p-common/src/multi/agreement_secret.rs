use super::{agreement_key::Idp2pAgreementKey, error::Idp2pMultiError};
use x25519_dalek::StaticSecret;

#[derive(Debug, Clone, PartialEq)]
pub enum Idp2pAgreementSecret {
    X25519 { secret: [u8;32]},
    Kyber512{ secret: [u8;1632]}
}

impl Idp2pAgreementSecret {
    pub fn to_agreement_key(&self) -> Idp2pAgreementKey {
        match self {
            Self::X25519 { secret } => {
                let static_secret = x25519_dalek::StaticSecret::from(*secret);
                let public_key = x25519_dalek::PublicKey::from(&static_secret);
                Idp2pAgreementKey::X25519 { public: public_key.to_bytes() }
            }
            Self::Kyber512 { secret } => todo!()
        }
    }

    pub fn to_shared_key(&self, agree_key: Idp2pAgreementKey) -> Result<Vec<u8>, Idp2pMultiError>{
        match self {
            Self::X25519 { secret } => {
                match agree_key {
                    Idp2pAgreementKey::X25519 { public } => {
                        let sender_secret = StaticSecret::from(*secret);
                        let pk = x25519_dalek::PublicKey::try_from(public)?;
                        let shared_secret = sender_secret.diffie_hellman(&pk);
                        Ok(shared_secret.to_bytes().to_vec())
                    }
                    _ => Err(Idp2pMultiError::InvalidKeyCode)
                }
            },
            Self::Kyber512 { secret } => todo!()
        }
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        match self {
            Self::X25519 { secret } => secret.to_vec(),
            Self::Kyber512 { secret } => secret.to_vec()
        }
    }
}
