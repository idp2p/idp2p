use super::{Signer, Verifier};
use crate::multi::error::Idp2pMultiError;
use pqcrypto_dilithium::ffi::*;

#[derive(PartialEq, Clone, Debug)]
pub struct Dilithium3Keypair {
    secret: [u8; PQCLEAN_DILITHIUM3_CLEAN_CRYPTO_SECRETKEYBYTES],
    public: [u8; PQCLEAN_DILITHIUM3_CLEAN_CRYPTO_PUBLICKEYBYTES],
}

#[derive(PartialEq, Clone, Debug)]
pub struct Dilithium3PublicKey([u8; PQCLEAN_DILITHIUM3_CLEAN_CRYPTO_PUBLICKEYBYTES]);

impl Verifier for Dilithium3PublicKey {
    fn pub_bytes(&self) -> Vec<u8> {
        self.0.to_vec()
    }

    fn verify(&self, payload: &[u8], sig: &[u8]) -> Result<bool, Idp2pMultiError> {
        let res = unsafe {
            PQCLEAN_DILITHIUM3_CLEAN_crypto_sign_verify(
                sig.as_ptr(),
                sig.len(),
                payload.as_ptr(),
                payload.len(),
                self.0.as_ptr(),
            )
        };
        Ok(res == 0)
    }
}

impl Signer for Dilithium3Keypair {
    type PublicKeyType = Dilithium3PublicKey;

    fn priv_bytes(&self) -> Vec<u8> {
        self.secret.to_vec()
    }

    fn to_public_key(&self) -> Self::PublicKeyType {
        Dilithium3PublicKey(self.public)
    }

    fn sign(&self, payload: &[u8]) -> Result<Vec<u8>, Idp2pMultiError> {
        let mut sig = [0u8; PQCLEAN_DILITHIUM3_CLEAN_CRYPTO_BYTES];
        let mut smlen: usize = 0;
        unsafe {
            PQCLEAN_DILITHIUM3_CLEAN_crypto_sign(
                sig.as_mut_ptr(),
                &mut smlen as *mut usize,
                payload.as_ptr(),
                payload.len(),
                self.secret.as_ptr()
            )
        };
        Ok(sig.to_vec())
    }
}

impl Dilithium3Keypair {
    pub fn generate() -> Self {
        let mut pk = [0u8; PQCLEAN_DILITHIUM3_CLEAN_CRYPTO_PUBLICKEYBYTES];
        let mut sk = [0u8; PQCLEAN_DILITHIUM3_CLEAN_CRYPTO_SECRETKEYBYTES];
        assert_eq!(
            unsafe { PQCLEAN_DILITHIUM3_CLEAN_crypto_sign_keypair(pk.as_mut_ptr(), sk.as_mut_ptr()) },
            0,
        );
        Self::new(sk, pk)
    }
    pub fn new(
        secret: [u8; PQCLEAN_DILITHIUM3_CLEAN_CRYPTO_SECRETKEYBYTES],
        public: [u8; PQCLEAN_DILITHIUM3_CLEAN_CRYPTO_PUBLICKEYBYTES],
    ) -> Self {
        Self {
            public: public,
            secret: secret,
        }
    }
}

impl From<[u8; PQCLEAN_DILITHIUM3_CLEAN_CRYPTO_PUBLICKEYBYTES]> for Dilithium3PublicKey{
    fn from(bytes: [u8; PQCLEAN_DILITHIUM3_CLEAN_CRYPTO_PUBLICKEYBYTES]) -> Self {
        Self(bytes)
    }
}

impl TryFrom<&[u8]> for Dilithium3PublicKey{
    type Error = Idp2pMultiError;

    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        let public: [u8; PQCLEAN_DILITHIUM3_CLEAN_CRYPTO_PUBLICKEYBYTES] = value.try_into()?;
        Ok(public.into())
    }
}

