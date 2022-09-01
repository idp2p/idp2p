use super::{
    error::Idp2pMultiError,
    verification::{
        dilithium3::{Dilithium3Keypair, Dilithium3PublicKey},
        ed25519::{Ed25519Keypair, Ed25519PublicKey},
        key_from_multi_bytes, key_to_multi_bytes, Signer, VerificationKeyCode, Verifier,
    },
};

#[derive(PartialEq, Clone, Debug)]
pub enum Idp2pAuthenticationKeypair {
    Ed25519(Ed25519Keypair),
    Dilithium3(Dilithium3Keypair),
}

#[derive(PartialEq, Clone, Debug)]
pub enum Idp2pAuthenticationPublicKey {
    Ed25519(Ed25519PublicKey),
    Dilithium3(Dilithium3PublicKey),
}

impl Idp2pAuthenticationKeypair {
    pub fn to_public_key(&self) -> Idp2pAuthenticationPublicKey {
        match &self {
            Idp2pAuthenticationKeypair::Ed25519(pk) => {
                Idp2pAuthenticationPublicKey::Ed25519(pk.to_public_key())
            }
            Idp2pAuthenticationKeypair::Dilithium3(pk) => {
                Idp2pAuthenticationPublicKey::Dilithium3(pk.to_public_key())
            }
        }
    }

    pub fn sign(&self, payload: &[u8]) -> Result<Vec<u8>, Idp2pMultiError> {
        match &self {
            Idp2pAuthenticationKeypair::Ed25519(keypair) => keypair.sign(payload),
            Idp2pAuthenticationKeypair::Dilithium3(keypair) => keypair.sign(payload),
        }
    }
}

impl Idp2pAuthenticationPublicKey {
    pub fn from_multi_bytes(bytes: &[u8]) -> Result<Self, Idp2pMultiError> {
        let (code, public) = key_from_multi_bytes(bytes)?;
        match code {
            VerificationKeyCode::Ed25519 => Ok(Self::Ed25519((&*public).try_into()?)),
            VerificationKeyCode::Dilithium3 => Ok(Self::Dilithium3((&*public).try_into()?)),
            _ => Err(Idp2pMultiError::InvalidKeyCode),
        }
    }

    // Serialize to bytes
    pub fn to_multi_bytes(&self) -> Result<Vec<u8>, Idp2pMultiError> {
        match &self {
            Self::Ed25519(pk) => key_to_multi_bytes(VerificationKeyCode::Ed25519, &pk.pub_bytes()),
            Self::Dilithium3(pk) => {
                key_to_multi_bytes(VerificationKeyCode::Dilithium3, &pk.pub_bytes())
            }
        }
    }
    pub fn to_bytes(&self) -> Vec<u8> {
        match &self {
            Self::Ed25519(pk) => pk.pub_bytes(),
            Self::Dilithium3(pk) => pk.pub_bytes(),
        }
    }
    // Verify payload with signature
    pub fn verify(&self, payload: &[u8], sig: &[u8]) -> Result<bool, Idp2pMultiError> {
        match &self {
            Self::Ed25519(pk) => pk.verify(payload, sig),
            Self::Dilithium3(pk) =>  pk.verify(payload, sig),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn sign_verify_test() -> Result<(), Idp2pMultiError> {
        let keypair = Idp2pAuthenticationKeypair::Ed25519(Ed25519Keypair::from_secret([0u8; 32]));
        let pk = keypair.to_public_key();
        let payload = vec![0u8; 10];
        let sig = keypair.sign(&payload)?;
        let r = pk.verify(&payload, &sig)?;
        assert!(r);
        Ok(())
    }
}
