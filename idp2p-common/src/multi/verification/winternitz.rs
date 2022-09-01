use super::{Signer, Verifier};
use crate::{multi::error::Idp2pMultiError, random::create_random};

#[derive(PartialEq, Clone, Debug)]
pub struct WinternitzKeypair {
    secret: [u8; winternitz::PRIVKEY_SIZE],
    public: [u8; winternitz::PUBKEY_SIZE],
}

#[derive(PartialEq, Clone, Debug)]
pub struct WinternitzPublicKey([u8; winternitz::PUBKEY_SIZE]);

impl Verifier for WinternitzPublicKey {
    fn pub_bytes(&self) -> Vec<u8> {
        self.0.to_vec()
    }
    fn verify(&self, payload: &[u8], sig: &[u8]) -> Result<bool, Idp2pMultiError> {
        let result = winternitz::verify(&self.0, payload, &sig).unwrap();
        Ok(result)
    }
}

impl Signer for WinternitzKeypair {
    type PublicKeyType = WinternitzPublicKey;

    fn priv_bytes(&self) -> Vec<u8> {
        self.secret.to_vec()
    }

    fn to_public_key(&self) -> Self::PublicKeyType {
        WinternitzPublicKey(self.public)
    }

    fn sign(&self, payload: &[u8]) -> Result<Vec<u8>, Idp2pMultiError> {
        let mut sig = [0; winternitz::SIG_SIZE];
        winternitz::sign(&self.secret, payload, &mut sig).unwrap();
        Ok(sig.to_vec())
    }
}

impl WinternitzKeypair {
    pub fn generate() -> Self {
        let sk = create_random::<{ winternitz::PRIVKEY_SIZE }>();
        let mut pk = [0; winternitz::PUBKEY_SIZE];
        winternitz::derive_pubkey(&sk, &mut pk).unwrap();
        Self {
            secret: sk,
            public: pk,
        }
    }
    pub fn new(
        secret: [u8; winternitz::PRIVKEY_SIZE],
        public: [u8; winternitz::PUBKEY_SIZE],
    ) -> Self {
        Self {
            public: public,
            secret: secret,
        }
    }
}


impl From<[u8; winternitz::PUBKEY_SIZE]> for WinternitzPublicKey{
    fn from(bytes: [u8; winternitz::PUBKEY_SIZE]) -> Self {
        Self(bytes)
    }
}

impl TryFrom<&[u8]> for WinternitzPublicKey{
    type Error = Idp2pMultiError;

    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        let public: [u8; winternitz::PUBKEY_SIZE] = value.try_into()?;
        Ok(public.into())
    }
}
