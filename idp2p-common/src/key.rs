use std::str::FromStr;

#[derive(PartialEq, Debug, Clone)]
pub enum Idp2pKeyDigest {
    Idp2pEd25519 { digest: [u8; 32] },
}

#[derive(PartialEq, Debug, Clone)]
pub enum Idp2pKey {
    Idp2pEd25519 { public: ed25519_dalek::PublicKey },
}

#[derive(PartialEq, Debug, Clone)]
pub enum Idp2pAgreementKey {
    Idp2pX25519 { public: x25519_dalek::PublicKey },
}

impl Idp2pKey {
    pub fn get_public(&self) -> Vec<u8> {
        match &self {
            Self::Idp2pEd25519 { public } => public.to_bytes().to_vec(),
        }
    }

    pub fn get_alg(&self) -> i32 {
        match &self {
            Self::Idp2pEd25519 { public: _ } => 0,
        }
    }
}

/*impl FromStr for Idp2pAgreementKey {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (_, key_data) = multibase::decode(&s).unwrap();
        Ok(key_data.try_into().unwrap())
    }
}

impl From<Vec<u8>> for Idp2pAgreementKey {
    fn from(data: Vec<u8>) -> Self {
        let key_type = data[0];
        match key_type {
            0 => {
                let bytes: [u8; 32] = data[1..].try_into().expect("Key should be 32 bytes");
                Self::Idp2pX25519 {
                    public: bytes.try_into().expect("Key should be 32 bytes"),
                }
            }
            _ => panic!("Not supported"),
        }
    }
}

impl Serialize for Idp2pAgreementKey {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match &self {
            Idp2pAgreementKey::Idp2pX25519 { public } => {
                let mut bytes = [0u8; 33];
                bytes[1..].copy_from_slice(public.as_bytes());
                let s = multibase::encode(multibase::Base::Base64Url, bytes);
                serializer.serialize_str(&s)
            }
        }
    }
}

impl<'de> Deserialize<'de> for Idp2pAgreementKey {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s: &str = Deserialize::deserialize(deserializer)?;
        Ok(Idp2pAgreementKey::from_str(s).unwrap())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::{Deserialize, Serialize};

    #[derive(PartialEq, Debug, Clone, Serialize, Deserialize)]
    struct Sample {
        agreement_key: Idp2pAgreementKey,
    }
    #[test]
    fn agreement_key_test() {
        let s = Sample {
            agreement_key: Idp2pAgreementKey::Idp2pX25519 {
                public: x25519_dalek::PublicKey::from([1u8; 32]),
            },
        };
        let str = serde_json::to_string(&s).unwrap();
        let s_a: Sample = serde_json::from_str(&str).unwrap();
        eprintln!("{str} {:?}", s_a.agreement_key);
        let mut one = [0u8;33];
        one[1..].copy_from_slice(&[1u8;32]);
        let one_str = multibase::encode(multibase::Base::Base64Url, one);
        eprintln!("{one_str}");
    }
}*/
