use cid::multihash::Multihash;

use super::{
    error::Idp2pMultiError,
    hash::Idp2pHasher,
    verification::{
        ed25519::{Ed25519Keypair, Ed25519PublicKey},
        key_from_bytes, key_to_bytes, VerificationKeyCode,
    },
};

#[derive(PartialEq, Clone, Debug)]
pub enum Idp2pLedgerKeypair {
    Ed25519(Ed25519Keypair),
    Dilithium3(),
    Winternitz(),
}

#[derive(PartialEq, Clone, Debug)]
pub enum Idp2pLedgerPublicKey {
    Ed25519(Ed25519PublicKey),
    Dilithium3(),
    Winternitz(),
}

#[derive(PartialEq, Clone, Debug)]
pub struct Idp2pLedgerPublicDigest {
    code: VerificationKeyCode,
    digest: Multihash,
}

impl Idp2pLedgerKeypair {
    pub fn new_ed25519(secret: [u8; 32]) -> Result<Self, Idp2pMultiError> {
        Ok(Self::Ed25519(Ed25519Keypair::from_secret(secret)?))
    }

    pub fn to_public_key(&self) {}

    pub fn sign(&self, payload: &[u8]) -> Result<Vec<u8>, Idp2pMultiError> {
        todo!()
    }
}

impl Idp2pLedgerPublicKey {
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, Idp2pMultiError> {
        let (code, public) = key_from_bytes(bytes)?;
        match code {
            VerificationKeyCode::Ed25519 => {
                let public: [u8; 32] = (&*public).try_into()?;
                Ok(Self::Ed25519(Ed25519PublicKey::from_bytes(public)))
            }
            VerificationKeyCode::Dilithium3 => todo!(),
            VerificationKeyCode::Winternitz => todo!(),
        }
    }

    // Serialize to bytes
    pub fn to_bytes(&self) -> Result<Vec<u8>, Idp2pMultiError> {
        match &self {
            Self::Ed25519(public) => todo!(),
            Self::Dilithium3() => todo!(),
            Self::Winternitz() => todo!(),
        }
        //key_to_bytes(self.c, bytes)
    }
    // To key digest
    pub fn to_digest(&self) {}
    // Verify payload with signature
    pub fn verify(&self, payload: &[u8], sig: &[u8]) {
        todo!()
    }
}

impl Idp2pLedgerPublicDigest {
    pub fn from_bytes() {}
    pub fn to_bytes(&self) {}
    pub fn ensure_public(&self, public: &[u8]) -> Result<(), Idp2pMultiError> {
        let hasher = Idp2pHasher::try_from(self.digest.code())?;
        hasher.ensure(self.digest, public)?;
        Ok(())
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
