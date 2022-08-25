pub mod ed25519;
use self::ed25519::*;

use super::error::Idp2pMultiError;
use cid::multihash::Multihash;

#[derive(PartialEq, Clone, Debug)]
pub enum Idp2pVerKeyCode {
    Ed25519 = 0xed,
    Dilithium2 = 0x1207,
    Winternitz = 0x1208,
}

const ED_U64: u64 = Idp2pVerKeyCode::Ed25519 as u64;
const DILITHIUM2_U64: u64 = Idp2pVerKeyCode::Dilithium2 as u64;
const WINTERNITZ_U64: u64 = Idp2pVerKeyCode::Winternitz as u64;

impl TryFrom<u64> for Idp2pVerKeyCode {
    type Error = Idp2pMultiError;

    fn try_from(value: u64) -> Result<Idp2pVerKeyCode, Self::Error> {
        match value {
            ED_U64 => Ok(Self::Ed25519),
            DILITHIUM2_U64 => Ok(Self::Dilithium2),
            WINTERNITZ_U64 => Ok(Self::Winternitz),
            _ => Err(Idp2pMultiError::InvalidKeyCode),
        }
    }
}

impl Idp2pVerKeyCode {
    pub fn assertion_key(&self, bytes: &[u8]) -> Result<Idp2pVerPubKey, Idp2pMultiError> {
        match &self {
            Self::Ed25519 | Self::Dilithium2 => Ok(Idp2pVerPubKey {
                code: self.to_owned(),
                public: bytes.to_owned(),
            }),
            _ => Err(Idp2pMultiError::InvalidKeyCode),
        }
    }
    pub fn authentication_key(&self, bytes: &[u8]) -> Result<Idp2pVerPubKey, Idp2pMultiError> {
        match &self {
            Self::Ed25519 | Self::Dilithium2 => Ok(Idp2pVerPubKey {
                code: self.to_owned(),
                public: bytes.to_owned(),
            }),
            _ => Err(Idp2pMultiError::InvalidKeyCode),
        }
    }

    pub fn ledger_key(&self, bytes: &[u8]) -> Result<Idp2pVerPubKey, Idp2pMultiError> {
        Ok(Idp2pVerPubKey {
            code: self.to_owned(),
            public: bytes.to_owned(),
        })
    }
}

pub trait Idp2pVerifier {
    fn verify(&self);
}

pub trait Idp2pSigner {
    fn verify(&self);
}
// Verification public key: [code][size][public bytes]
#[derive(PartialEq, Clone, Debug)]
pub struct Idp2pVerPubKey {
    code: Idp2pVerKeyCode,
    public: Vec<u8>,
}

impl Idp2pVerPubKey {
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
        match self.code {
            Idp2pVerKeyCode::Ed25519 =>  Ed25519Provider.verify(),
            Idp2pVerKeyCode::Dilithium2 => todo!(),
            Idp2pVerKeyCode::Winternitz => todo!(),
        }
    }
}

#[derive(PartialEq, Clone, Debug)]
pub struct Idp2pVerPubKeySecret {
    code: Idp2pVerKeyCode,
    secret: Vec<u8>,
}

#[derive(PartialEq, Clone, Debug)]
pub struct Idp2pVerPubKeyDigest {
    code: Idp2pVerKeyCode,
    digest: Multihash,
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn try_from_test() -> Result<(), Idp2pMultiError> {
        let v: u64 = 0xec;
        let key_code: Idp2pVerKeyCode = v.try_into()?;
        matches!(key_code, Idp2pVerKeyCode::Ed25519);
        Ok(())
    }
}
