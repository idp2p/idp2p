use crate::multi::error::Idp2pMultiError;

use super::{AgreementPublicBehaviour, AgreementSecretBehaviour, AgreementShared};
use pqcrypto_kyber::ffi::*;

#[derive(PartialEq, Clone, Debug)]
pub struct Kyber768PublicKey(pub(super) [u8; PQCLEAN_KYBER768_CLEAN_CRYPTO_PUBLICKEYBYTES]);

#[derive(PartialEq, Clone, Debug)]
pub struct Kyber768Keypair {
    secret: [u8; PQCLEAN_KYBER768_CLEAN_CRYPTO_SECRETKEYBYTES],
    public: [u8; PQCLEAN_KYBER768_CLEAN_CRYPTO_PUBLICKEYBYTES],
}

impl AgreementSecretBehaviour for Kyber768Keypair {
    type PublicKeyType = Kyber768PublicKey;
    fn priv_as_bytes<'a>(&'a self) -> &'a [u8] {
        &self.secret
    }

    fn to_public_key(&self) -> Kyber768PublicKey {
        Kyber768PublicKey(self.public)
    }

    fn resolve_shared_secret(&self, data: &[u8]) -> Result<Vec<u8>, Idp2pMultiError> {
        let mut ss = [0u8; PQCLEAN_KYBER768_CLEAN_CRYPTO_BYTES];
        assert_eq!(
            unsafe { PQCLEAN_KYBER768_CLEAN_crypto_kem_dec(ss.as_mut_ptr(), data.as_ptr(), self.secret.as_ptr()) },
            0,
        );

        Ok(ss.to_vec())
    }
}

impl Kyber768Keypair {
    pub fn generate() -> Self {
        let mut pk = [0u8; PQCLEAN_KYBER768_CLEAN_CRYPTO_PUBLICKEYBYTES];
        let mut sk = [0u8; PQCLEAN_KYBER768_CLEAN_CRYPTO_SECRETKEYBYTES];
        assert_eq!(
            unsafe { PQCLEAN_KYBER768_CLEAN_crypto_kem_keypair(pk.as_mut_ptr(), sk.as_mut_ptr()) },
            0,
        );
        Self::new(sk, pk)
    }
    pub fn new(
        secret: [u8; PQCLEAN_KYBER768_CLEAN_CRYPTO_SECRETKEYBYTES],
        public: [u8; PQCLEAN_KYBER768_CLEAN_CRYPTO_PUBLICKEYBYTES],
    ) -> Self {
        Self {
            public: public,
            secret: secret,
        }
    }
}

impl From<[u8; PQCLEAN_KYBER768_CLEAN_CRYPTO_PUBLICKEYBYTES]> for Kyber768PublicKey{
    fn from(value: [u8; PQCLEAN_KYBER768_CLEAN_CRYPTO_PUBLICKEYBYTES]) -> Self {
        Self(value)
    }
}

impl TryFrom<&[u8]> for Kyber768PublicKey{
    type Error = Idp2pMultiError;

    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        let public: [u8; PQCLEAN_KYBER768_CLEAN_CRYPTO_PUBLICKEYBYTES] = value.try_into()?;
        Ok(public.into())
    }
}

impl AgreementPublicBehaviour for Kyber768PublicKey {
    fn as_bytes<'a>(&'a self) -> &'a [u8] {
        &self.0
    }

    fn create_shared(&self) -> Result<AgreementShared, Idp2pMultiError> {
        let mut ss = [0u8; PQCLEAN_KYBER768_CLEAN_CRYPTO_BYTES];
        let mut ct = [0u8; PQCLEAN_KYBER768_CLEAN_CRYPTO_CIPHERTEXTBYTES];
        assert_eq!(
            unsafe {
                PQCLEAN_KYBER768_CLEAN_crypto_kem_enc(
                    ct.as_mut_ptr(),
                    ss.as_mut_ptr(),
                    self.0.as_ptr(),
                )
            },
            0,
        );
        Ok(AgreementShared {
            secret: ss.to_vec(),
            data: ct.to_vec(),
        })
    }
}