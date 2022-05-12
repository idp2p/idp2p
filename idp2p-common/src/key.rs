use anyhow::{bail, Result};
use ed25519_dalek::{PublicKey, Signature, Verifier};

use crate::base64url;

#[derive(PartialEq, Clone)]
pub enum Idp2pKey {
    Idp2pEd25519 { public: PublicKey },
}

impl std::fmt::Debug for Idp2pKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Idp2pEd25519 { public } => write!(
                f,
                "Ed25519({:?})",
                base64url::encode_bytes(&public.to_bytes()).unwrap()
            ),
        }
    }
}

impl From<Idp2pKey> for Vec<u8> {
    fn from(value: Idp2pKey) -> Self {
        match value {
            Idp2pKey::Idp2pEd25519 { public } => {
                public.as_bytes().to_vec()
            }
        }
    }
}

impl Idp2pKey {
    pub fn try_from(code: u64, public: &[u8]) -> Result<Self> {
        match code {
            0xed => Ok(Self::Idp2pEd25519 {
                public: PublicKey::from_bytes(public)?,
            }),
            _ => bail!(""),
        }
    }
    pub fn verify(&self, payload: &[u8], sig: &[u8]) -> Result<bool> {
        match self {
            Idp2pKey::Idp2pEd25519 { public } => {
                let sig_bytes: [u8; 64] = sig.try_into()?;
                let signature = Signature::from(sig_bytes);
                Ok(public.verify(payload, &signature).is_ok())
            }
        }
    }
}
/*
impl FromStr for Idp2pKey {
    type Err = anyhow::Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (_, key_data) = multibase::decode(&s).unwrap();
        Ok(key_data.try_into().unwrap())
    }
}

impl TryFrom<Vec<u8>> for Idp2pKey {
    type Error = anyhow::Error;

    fn try_from(value: Vec<u8>) -> Result<Self, Self::Error> {
        let key_type = value[0];
        match key_type {
            0 => {
                let bytes: [u8; 32] = value[1..].try_into()?;
                Ok(Self::Idp2pEd25519 {
                    public: ed25519_dalek::PublicKey::from_bytes(&bytes)?,
                })
            }
            _ => anyhow::bail!("Not supported"),
        }
    }
}

impl From<Idp2pKey> for Vec<u8> {
    fn from(value: Idp2pKey) -> Self {
        match value {
            Idp2pKey::Idp2pEd25519 { public } => {
                let mut bytes = [0u8; 33];
                bytes[1..].copy_from_slice(public.as_bytes());
                bytes.to_vec()
            }
        }
    }
}

impl Serialize for Idp2pKey {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let encoded: Vec<u8> = self.to_owned().into();
        let s = multibase::encode(multibase::Base::Base64Url, encoded);
        serializer.serialize_str(&s)
    }
}

impl<'de> Deserialize<'de> for Idp2pKey {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s: &str = Deserialize::deserialize(deserializer)?;
        Ok(Self::from_str(s).unwrap())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn try_from_test() {
        let mut bytes = [0u8; 33];
        bytes[1..].copy_from_slice(&[1u8; 32]);
        let key: Idp2pKey = bytes.to_vec().try_into().unwrap();
        assert!(
            matches!(key, Idp2pKey::Idp2pEd25519 { public } if public.as_bytes().to_owned() == [1u8; 32])
        );
    }

    #[test]
    fn encode_test() {
        let proof = Idp2pKey::Idp2pEd25519 {
            public: ed25519_dalek::PublicKey::from_bytes(&[0u8; 32]).unwrap(),
        };
        let vec: Vec<u8> = proof.into();
        assert_eq!(vec.len(), 33);
    }
}
 */
