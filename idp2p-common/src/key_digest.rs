use std::str::FromStr;

use serde::{Deserialize, Serialize};

#[derive(PartialEq, Debug, Clone)]
pub enum Idp2pKeyDigest {
    Idp2pEd25519Sha256 { digest: [u8; 32] },
}

impl FromStr for Idp2pKeyDigest {
    type Err = anyhow::Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (_, key_data) = multibase::decode(&s).unwrap();
        Ok(key_data.try_into().unwrap())
    }
}

impl TryFrom<Vec<u8>> for Idp2pKeyDigest {
    type Error = anyhow::Error;

    fn try_from(value: Vec<u8>) -> Result<Self, Self::Error> {
        let key_type = value[0];
        match key_type {
            0 => {
                let bytes: [u8; 32] = value[1..].try_into()?;
                Ok(Self::Idp2pEd25519Sha256 {
                    digest: bytes.try_into()?,
                })
            }
            _ => anyhow::bail!("Not supported"),
        }
    }
}

impl From<Idp2pKeyDigest> for Vec<u8> {
    fn from(value: Idp2pKeyDigest) -> Self {
        match value {
            Idp2pKeyDigest::Idp2pEd25519Sha256 { digest } => {
                let mut bytes = [0u8; 33];
                bytes[1..].copy_from_slice(&digest);
                bytes.to_vec()
            }
        }
    }
}

impl Serialize for Idp2pKeyDigest {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let encoded: Vec<u8> = self.to_owned().into();
        let s = multibase::encode(multibase::Base::Base64Url, encoded);
        serializer.serialize_str(&s)
    }
}

impl<'de> Deserialize<'de> for Idp2pKeyDigest {
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
        let digest: Idp2pKeyDigest = bytes.to_vec().try_into().unwrap();
        assert!(
            matches!(digest, Idp2pKeyDigest::Idp2pEd25519Sha256 { digest } if digest == [1u8; 32])
        );
    }

    #[test]
    fn encode_test() {
        let proof = Idp2pKeyDigest::Idp2pEd25519Sha256 { digest: [0u8;32] };
        let vec: Vec<u8> = proof.into();
        assert_eq!(vec.len(), 33);
    }
}
