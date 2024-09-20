use std::io::Read;

use super::{
    error::Idp2pMultiError,
    verification::{
        dilithium3::{Dilithium3Keypair, Dilithium3PublicKey},
        ed25519::{Ed25519Keypair, Ed25519PublicKey},
        key_from_multi_bytes, key_to_multi_bytes,
        winternitz::{WinternitzKeypair, WinternitzPublicKey},
        Signer, VerificationKeyCode, Verifier,
    }, hash::Idp2pMultiHash,
};
use unsigned_varint::{encode as varint_encode, io::read_u64};

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
    pub fn new(code: VerificationKeyCode, pk: &[u8]) -> Result<Self, Idp2pMultiError> {
        match code {
            VerificationKeyCode::Ed25519 => Ok(Self::Ed25519(pk.try_into()?)),
            VerificationKeyCode::Dilithium3 => Ok(Self::Dilithium3(pk.try_into()?)),
            VerificationKeyCode::Winternitz => Ok(Self::Winternitz(pk.try_into()?)),
        }
    }
    pub fn from_multi_bytes(bytes: &[u8]) -> Result<Self, Idp2pMultiError> {
        let (code, public) = key_from_multi_bytes(bytes)?;
        match code {
            VerificationKeyCode::Ed25519 => Ok(Self::Ed25519((&*public).try_into()?)),
            VerificationKeyCode::Dilithium3 => Ok(Self::Dilithium3((&*public).try_into()?)),
            VerificationKeyCode::Winternitz => Ok(Self::Winternitz((&*public).try_into()?)),
        }
    }

    // Serialize to bytes
    pub fn to_multi_bytes(&self) -> Vec<u8> {
        match &self {
            Self::Ed25519(pk) => key_to_multi_bytes(VerificationKeyCode::Ed25519, pk.as_bytes()),
            Self::Dilithium3(pk) => {
                key_to_multi_bytes(VerificationKeyCode::Dilithium3, pk.as_bytes())
            }
            Self::Winternitz(pk) => {
                key_to_multi_bytes(VerificationKeyCode::Winternitz, pk.as_bytes())
            }
        }
    }
    pub fn as_bytes<'a>(&'a self) -> &'a [u8] {
        match &self {
            Self::Ed25519(pk) => pk.as_bytes(),
            Self::Dilithium3(pk) => pk.as_bytes(),
            Self::Winternitz(pk) => pk.as_bytes(),
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
    pub fn to_digest(&self) -> Result<Idp2pLedgerPublicHash, Idp2pMultiError> {
        let (code, pk) = match &self {
            Self::Ed25519(pk) => (VerificationKeyCode::Ed25519, pk.as_bytes()),
            Self::Dilithium3(pk) => (VerificationKeyCode::Dilithium3, pk.as_bytes()),
            Self::Winternitz(pk) => (VerificationKeyCode::Winternitz, pk.as_bytes()),
        };
        let hash = Idp2pMultiHash::new(pk)?;
        Ok(Idp2pLedgerPublicHash::new(code, hash))
    }
}

#[derive(PartialEq, Clone, Debug)]
pub struct Idp2pLedgerPublicHash {
    code: VerificationKeyCode,
    hash: Idp2pMultiHash,
}

impl Idp2pLedgerPublicHash {
    pub fn new(code: VerificationKeyCode, hash: Idp2pMultiHash) -> Self {
        Self {
            code: code,
            hash: hash,
        }
    }

    pub fn code(&self) -> VerificationKeyCode {
        self.code.clone()
    }
    
    pub fn from_multi_bytes(bytes: &[u8]) -> Result<Idp2pLedgerPublicHash, Idp2pMultiError> {
        let mut r = bytes;
        let code: VerificationKeyCode = read_u64(&mut r)?.try_into()?;
        let mut mh_bytes: Vec<u8> = vec![];
        r.read_to_end(&mut mh_bytes)?;
        Ok(Self {
            code: code,
            hash: Idp2pMultiHash::from_bytes(&mh_bytes)?,
        })
    }

    pub fn to_multi_bytes(&self) -> Vec<u8> {
        let code = self.code.clone() as u64;
        let mut code_buf = varint_encode::u64_buffer();
        let code = varint_encode::u64(code, &mut code_buf);
        [code, &self.hash.to_bytes()].concat()
    }

    pub fn ensure_public(&self, public: &[u8]) -> Result<(), Idp2pMultiError> {
        self.hash.ensure(public)
    }
}

#[cfg(test)]
mod tests {
    use crate::multi::verification::winternitz::WinternitzKeypair;

    use super::*;
    #[test]
    fn sign_verify_test() -> Result<(), Idp2pMultiError> {
        let keypair = Idp2pLedgerKeypair::Ed25519(Ed25519Keypair::generate());
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
