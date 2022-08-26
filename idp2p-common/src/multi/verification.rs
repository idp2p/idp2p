pub mod ed25519;

use self::ed25519::Ed25519PublicKey;

use super::{error::Idp2pMultiError, hash::Idp2pHasher};
use cid::multihash::Multihash;

const ED_U64: u64 = 0xed;
const DILITHIUM2_U64: u64 = 0x1207;
const WINTERNITZ_U64: u64 = 0x1208;

pub trait VerificationPublicKey {
    fn pub_bytes(&self) -> Vec<u8>;
    fn verify(&self, payload: &[u8], sig: &[u8]) -> Result<bool, Idp2pMultiError>;
}

pub trait VerificationKeypair {
    type PublicKeyType;
    fn priv_bytes(&self) -> Vec<u8>;
    fn to_public_key(&self) -> Self::PublicKeyType;
}

#[derive(PartialEq, Clone, Debug)]
pub enum Idp2pVerificationPublicKey {
    Ed25519(Ed25519PublicKey),
    Dilithium2(),
    Winternitz(),
}

#[derive(PartialEq, Clone, Debug)]
pub enum Idp2pVerificationKeypair {
    Ed25519(),
    Dilithium2(),
    Winternitz(),
}

#[derive(PartialEq, Clone, Debug)]
pub struct Idp2pVerificationPublicKeyDigest {
    code: u64,
    digest: Multihash,
}

impl Idp2pVerificationPublicKey {
    pub fn new_assertion_key(code: u64, bytes: &[u8]) -> Result<Self, Idp2pMultiError> {
        match code {
            ED_U64 => todo!(),
            DILITHIUM2_U64 => todo!(),
            WINTERNITZ_U64 => todo!(),
            _ => Err(Idp2pMultiError::InvalidKeyCode),
        }
    }
    pub fn new_next_key(code: u64, bytes: &[u8]) -> Result<Self, Idp2pMultiError> {
        match code {
            ED_U64 => todo!(),
            DILITHIUM2_U64 => todo!(),
            WINTERNITZ_U64 => todo!(),
            _ => Err(Idp2pMultiError::InvalidKeyCode),
        }
    }
    // Serialize to bytes
    pub fn encode() {}
    // Deserialize from bytes
    pub fn decode() {}
    // Produce id(hash)
    pub fn to_id() {}
    // To key digest
    pub fn to_digest() {}
    // Verify payload with signature
    pub fn verify(&self, payload: &[u8], sig: &[u8]) {
        todo!()
    }
}

impl Idp2pVerificationPublicKeyDigest {
    // Serialize to bytes
    pub fn encode() {}
    // Deserialize from bytes
    pub fn decode() {}
    // To key digest
    pub fn to_public_key(&self, p: &[u8]) -> Result<Idp2pVerificationPublicKey, Idp2pMultiError> {
        let hasher = Idp2pHasher::try_from(self.digest.code())?;
        hasher.ensure(self.digest, p)?;
        Idp2pVerificationPublicKey::new_next_key(self.code, p)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn try_from_test() -> Result<(), Idp2pMultiError> {
        Ok(())
    }
}
