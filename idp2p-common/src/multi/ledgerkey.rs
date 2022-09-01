use cid::multihash::Multihash;

use super::{
    error::Idp2pMultiError,
    hash::Idp2pHasher,
    verification::{
        dilithium3::{Dilithium3Keypair, Dilithium3PublicKey},
        ed25519::{Ed25519Keypair, Ed25519PublicKey},
        key_from_multi_bytes, key_to_multi_bytes,
        winternitz::{WinternitzKeypair, WinternitzPublicKey},
        Signer, VerificationKeyCode, Verifier,
    },
};
use unsigned_varint::{encode as varint_encode};

#[derive(PartialEq, Clone, Debug)]
pub enum Idp2pLedgerKeypair {
    Ed25519(Ed25519Keypair),
    Dilithium3(Dilithium3Keypair),
    Winternitz(WinternitzKeypair),
}

#[derive(PartialEq, Clone, Debug)]
pub enum Idp2pLedgerPublicKey {
    Ed25519(Ed25519PublicKey),
    Dilithium3(Dilithium3PublicKey),
    Winternitz(WinternitzPublicKey),
}

#[derive(PartialEq, Clone, Debug)]
pub struct Idp2pLedgerPublicDigest {
    code: VerificationKeyCode,
    digest: Multihash,
}

impl Idp2pLedgerKeypair {
    pub fn to_public_key(&self) -> Idp2pLedgerPublicKey {
        match &self {
            Idp2pLedgerKeypair::Ed25519(pk) => Idp2pLedgerPublicKey::Ed25519(pk.to_public_key()),
            Idp2pLedgerKeypair::Dilithium3(pk) => {
                Idp2pLedgerPublicKey::Dilithium3(pk.to_public_key())
            }
            Idp2pLedgerKeypair::Winternitz(pk) => {
                Idp2pLedgerPublicKey::Winternitz(pk.to_public_key())
            }
        }
    }

    pub fn sign(&self, payload: &[u8]) -> Result<Vec<u8>, Idp2pMultiError> {
        match &self {
            Idp2pLedgerKeypair::Ed25519(keypair) => keypair.sign(payload),
            Idp2pLedgerKeypair::Dilithium3(keypair) => keypair.sign(payload),
            Idp2pLedgerKeypair::Winternitz(keypair) => keypair.sign(payload),
        }
    }
}

impl Idp2pLedgerPublicKey {
    pub fn from_multi_bytes(bytes: &[u8]) -> Result<Self, Idp2pMultiError> {
        let (code, public) = key_from_multi_bytes(bytes)?;
        match code {
            VerificationKeyCode::Ed25519 => Ok(Self::Ed25519((&*public).try_into()?)),
            VerificationKeyCode::Dilithium3 => Ok(Self::Dilithium3((&*public).try_into()?)),
            VerificationKeyCode::Winternitz => Ok(Self::Winternitz((&*public).try_into()?)),
        }
    }

    // Serialize to bytes
    pub fn to_multi_bytes(&self) -> Result<Vec<u8>, Idp2pMultiError> {
        match &self {
            Self::Ed25519(pk) => {
                key_to_multi_bytes(VerificationKeyCode::Ed25519, &pk.pub_bytes())
            }
            Self::Dilithium3(pk) => todo!(),
            Self::Winternitz(pk) => todo!(),
        }
    }
    pub fn to_bytes(&self) -> Vec<u8> {
        match &self {
            Self::Ed25519(pk) => pk.pub_bytes(),
            Self::Dilithium3(pk) => pk.pub_bytes(),
            Self::Winternitz(pk) => pk.pub_bytes(),
        }
    }
    /// Verify payload with signature
    pub fn verify(&self, payload: &[u8], sig: &[u8]) -> Result<bool, Idp2pMultiError> {
        match &self {
            Self::Ed25519(pk) => pk.verify(payload, sig),
            Self::Dilithium3(pk) => pk.verify(payload, sig),
            Self::Winternitz(pk) => pk.verify(payload, sig),
        }
    }

    // To key digest
    pub fn to_digest(&self) -> Result<Idp2pLedgerPublicDigest, Idp2pMultiError> {
        todo!()
    }
}

impl Idp2pLedgerPublicDigest {
    pub fn from_multi_bytes() {}
    pub fn to_multi_bytes(&self) -> Vec<u8> {
        let code = self.code.clone() as u64;
        let mut code_buf = varint_encode::u64_buffer();
        let code = varint_encode::u64(code, &mut code_buf);
        [code, &self.digest.to_bytes()].concat()
    }
    pub fn ensure_public(&self, public: &[u8]) -> Result<(), Idp2pMultiError> {
        let hasher = Idp2pHasher::try_from(self.digest.code())?;
        hasher.ensure(self.digest, public)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::multi::verification::winternitz::WinternitzKeypair;

    use super::*;
    #[test]
    fn sign_verify_test() -> Result<(), Idp2pMultiError> {
        let keypair = Idp2pLedgerKeypair::Ed25519(Ed25519Keypair::from_secret([0u8; 32]));
        let pk = keypair.to_public_key();
        let payload = vec![0u8; 10];
        let sig = keypair.sign(&payload)?;
        let r = pk.verify(&payload, &sig)?;
        assert!(r);
        Ok(())
    }

    #[test]
    fn winternitz_sign_verify_test() -> Result<(), Idp2pMultiError> {
        let keypair = Idp2pLedgerKeypair::Winternitz(WinternitzKeypair::generate());
        let pk = keypair.to_public_key();
        let payload = vec![0u8; 10];
        let sig = keypair.sign(&payload)?;
        let r = pk.verify(&payload, &sig)?;
        assert!(r);
        Ok(())
    }
}
