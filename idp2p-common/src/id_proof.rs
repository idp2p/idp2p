use std::str::FromStr;

use serde::{Deserialize, Serialize};

#[derive(PartialEq, Debug, Clone)]
pub enum Idp2pProof {
    Sha256 { key: [u8; 32], value: [u8; 32] },
}

impl FromStr for Idp2pProof {
    type Err = anyhow::Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (_, data) = multibase::decode(&s)?;
        Ok(data.try_into()?)
    }
}

impl TryFrom<Vec<u8>> for Idp2pProof {
    type Error = anyhow::Error;

    fn try_from(value: Vec<u8>) -> Result<Self, Self::Error> {
        let key_type = value[0];
        match key_type {
            0 => Ok(Self::Sha256 {
                key: value[1..33].try_into()?,
                value: value[33..].try_into()?,
            }),
            _ => anyhow::bail!("Not supported"),
        }
    }
}

impl From<Idp2pProof> for Vec<u8> {
    fn from(value: Idp2pProof) -> Self {
        match value {
            Idp2pProof::Sha256 { key, value } => {
                let mut bytes = [0u8; 65];
                bytes[1..33].copy_from_slice(&key);
                bytes[33..].copy_from_slice(&value);
                bytes.to_vec()
            }
        }
    }
}

impl Serialize for Idp2pProof {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let encoded: Vec<u8> = self.to_owned().into();
        let s = multibase::encode(multibase::Base::Base64Url, encoded);
        serializer.serialize_str(&s)
    }
}

impl<'de> Deserialize<'de> for Idp2pProof {
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
        let mut bytes = [0u8; 65];
        bytes[1..33].copy_from_slice(&[1u8; 32]);
        bytes[33..].copy_from_slice(&[2u8; 32]);
        let proof: Idp2pProof = bytes.to_vec().try_into().unwrap();
        assert!(
            matches!(proof, Idp2pProof::Sha256 { key, value } if key == [1u8; 32] && value == [2u8; 32])
        );
    }
    
    #[test]
    fn encode_test() {
        let proof = Idp2pProof::Sha256 {
            key: [1u8; 32],
            value: [2u8; 32],
        };
        let vec: Vec<u8> = proof.into();
        assert_eq!(vec.len(), 65);
    }
}
