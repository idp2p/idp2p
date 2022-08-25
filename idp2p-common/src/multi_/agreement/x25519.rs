use crate::multi_::error::Idp2pMultiError;

use super::{AgreementPublicKey, AgreementShared};

use rand::rngs::OsRng;
use x25519_dalek::{EphemeralSecret, PublicKey};

#[derive(PartialEq, Clone, Debug)]
pub struct X25519PublicKey(pub(super) [u8;32]);

impl X25519PublicKey{
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, Idp2pMultiError>{
        Ok(Self(bytes.try_into()?))
    }
}

impl AgreementPublicKey for X25519PublicKey{
    fn pub_bytes(&self) -> Vec<u8> {
        todo!()
    }

    fn create_data(&self) -> Result<AgreementShared, Idp2pMultiError> {
        /*let ephemeral_secret = EphemeralSecret::new(OsRng);
        let ephemeral_public = PublicKey::from(&ephemeral_secret);
        let ephemeral_key = Idp2pAgreementPublicKey::X25519 {
            public: ephemeral_public.to_bytes(),
        };
        let pk = PublicKey::try_from(public)?;
        let shared_secret = ephemeral_secret.diffie_hellman(&pk);
        Ok((shared_secret.to_bytes().to_vec(), ephemeral_key.encode()))*/
        todo!()
    }
}