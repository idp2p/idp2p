use cid::multihash::Multihash;

use super::{
    error::Idp2pMultiError,
    hash::Idp2pHasher,
    verification::{
        ed25519::{Ed25519Keypair, Ed25519PublicKey},
        key_from_multi_bytes, key_to_multi_bytes, Signer, VerificationKeyCode, Verifier,
    },
};
use unsigned_varint::{encode as varint_encode, io::read_u64};

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

    pub fn to_public_key(&self) -> Idp2pLedgerPublicKey {
        match &self {
            Idp2pLedgerKeypair::Ed25519(ed_pub) => {
                Idp2pLedgerPublicKey::Ed25519(ed_pub.to_public_key())
            }
            Idp2pLedgerKeypair::Dilithium3() => todo!(),
            Idp2pLedgerKeypair::Winternitz() => todo!(),
        }
    }

    pub fn sign(&self, payload: &[u8]) -> Result<Vec<u8>, Idp2pMultiError> {
        match &self {
            Idp2pLedgerKeypair::Ed25519(keypair) => keypair.sign(payload),
            Idp2pLedgerKeypair::Dilithium3() => todo!(),
            Idp2pLedgerKeypair::Winternitz() => todo!(),
        }
    }
}

impl Idp2pLedgerPublicKey {
    pub fn from_multi_bytes(bytes: &[u8]) -> Result<Self, Idp2pMultiError> {
        let (code, public) = key_from_multi_bytes(bytes)?;
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
    pub fn to_multi_bytes(&self) -> Result<Vec<u8>, Idp2pMultiError> {
        match &self {
            Self::Ed25519(public) => {
                key_to_multi_bytes(VerificationKeyCode::Ed25519, &public.pub_bytes())
            }
            Self::Dilithium3() => todo!(),
            Self::Winternitz() => todo!(),
        }
    }
    pub fn to_bytes(&self) -> Vec<u8> {
        match &self {
            Self::Ed25519(public) => public.pub_bytes(),
            Self::Dilithium3() => todo!(),
            Self::Winternitz() => todo!(),
        }
    }
    /// Verify payload with signature
    pub fn verify(&self, payload: &[u8], sig: &[u8]) -> Result<bool, Idp2pMultiError> {
        match &self {
            Self::Ed25519(public) => public.verify(payload, sig),
            Self::Dilithium3() => todo!(),
            Self::Winternitz() => todo!(),
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
    use super::*;
    #[test]
    fn try_from_test() -> Result<(), Idp2pMultiError> {
        Ok(())
    }
}
