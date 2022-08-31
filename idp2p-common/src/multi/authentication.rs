use super::{
    error::Idp2pMultiError,
    verification::{
        ed25519::{Ed25519Keypair, Ed25519PublicKey},
        key_from_multi_bytes, key_to_multi_bytes, Signer, VerificationKeyCode, Verifier,
    },
};

#[derive(PartialEq, Clone, Debug)]
pub enum Idp2pAuthenticationKeypair {
    Ed25519(Ed25519Keypair),
    Dilithium3(),
}

#[derive(PartialEq, Clone, Debug)]
pub enum Idp2pAuthenticationPublicKey {
    Ed25519(Ed25519PublicKey),
    Dilithium3(),
}

impl Idp2pAuthenticationKeypair {
    pub fn new_ed25519(secret: [u8; 32]) -> Result<Self, Idp2pMultiError> {
        Ok(Self::Ed25519(Ed25519Keypair::from_secret(secret)?))
    }

    pub fn to_public_key(&self) -> Idp2pAuthenticationPublicKey {
        match &self {
            Idp2pAuthenticationKeypair::Ed25519(ed_pub) => {
                Idp2pAuthenticationPublicKey::Ed25519(ed_pub.to_public_key())
            }
            Idp2pAuthenticationKeypair::Dilithium3() => todo!(),
        }
    }

    pub fn sign(&self, payload: &[u8]) -> Result<Vec<u8>, Idp2pMultiError> {
        match &self {
            Idp2pAuthenticationKeypair::Ed25519(keypair) => keypair.sign(payload),
            Idp2pAuthenticationKeypair::Dilithium3() => todo!(),
        }
    }
}

impl Idp2pAuthenticationPublicKey {
    pub fn from_multi_bytes(bytes: &[u8]) -> Result<Self, Idp2pMultiError> {
        let (code, public) = key_from_multi_bytes(bytes)?;
        match code {
            VerificationKeyCode::Ed25519 => {
                let public: [u8; 32] = (&*public).try_into()?;
                Ok(Self::Ed25519(Ed25519PublicKey::from_bytes(public)))
            }
            VerificationKeyCode::Dilithium3 => todo!(),
            _ => Err(Idp2pMultiError::InvalidKeyCode),
        }
    }

    // Serialize to bytes
    pub fn to_multi_bytes(&self) -> Result<Vec<u8>, Idp2pMultiError> {
        match &self {
            Self::Ed25519(public) => {
                key_to_multi_bytes(VerificationKeyCode::Ed25519, &public.pub_bytes())
            }
            Self::Dilithium3() => todo!(),
        }
        //key_to_bytes(self.c, bytes)
    }
    pub fn to_bytes(&self) -> Vec<u8> {
        match &self {
            Self::Ed25519(public) => public.pub_bytes(),
            Self::Dilithium3() => todo!(),
        }
    }
    // Verify payload with signature
    pub fn verify(&self, payload: &[u8], sig: &[u8]) -> Result<bool, Idp2pMultiError> {
        match &self {
            Self::Ed25519(public) => public.verify(payload, sig),
            Self::Dilithium3() => todo!(),
        }
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
