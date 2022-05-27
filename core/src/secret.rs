use crate::keys::{agreement_key::Idp2pAgreementKey, key::Idp2pKey};

#[derive(Debug)]
pub enum Idp2pSecret{
    Ed25519{
        secret: ed25519_dalek::SecretKey
    }
}

impl Idp2pSecret{
    pub fn sign(){

    }
    pub fn to_shared_secret(&self, public: [u8; 32]) -> Vec<u8>{
       todo!()
    }
}

impl TryInto<Idp2pAgreementKey> for Idp2pSecret{
    type Error = crate::keys::error::MultiKeyError;

    fn try_into(self) -> Result<Idp2pAgreementKey, Self::Error> {
        match self{
            Idp2pSecret::Ed25519 { secret } => todo!(),
        }
    }
}

impl TryInto<Idp2pKey> for Idp2pSecret{
    type Error = crate::keys::error::MultiKeyError;

    fn try_into(self) -> Result<Idp2pKey, Self::Error> {
        match self{
            Idp2pSecret::Ed25519 { secret } => todo!(),
        }
    }
}