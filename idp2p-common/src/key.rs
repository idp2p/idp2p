use crate::{encode_vec, ED25519_CODE};
use anyhow::Result;
use ed25519_dalek::{PublicKey, Signature, Verifier};
use serde::{Deserialize, Serialize};

#[derive(PartialEq, Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum Idp2pKey {
    Idp2pEd25519 {
        #[serde(with = "encode_vec")]
        public: Vec<u8>,
    },
}

#[derive(PartialEq, Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum Idp2pAgreementKey {
    Idp2pX25519 {
        #[serde(with = "encode_vec")]
        public: Vec<u8>,
    },
}

impl Idp2pKey {
    pub fn new(code: u64, public: &[u8]) -> Result<Self> {
        match code {
            ED25519_CODE => Ok(Idp2pKey::Idp2pEd25519 {
                public: public.to_vec(),
            }),
            _ => anyhow::bail!("Invalid key code"),
        }
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        match &self {
            Idp2pKey::Idp2pEd25519 { public } => public.to_vec(),
        }
    }
    
    pub fn verify(&self, payload: &[u8], sig: &[u8]) -> Result<()> {
        match &self {
            Idp2pKey::Idp2pEd25519 { public } => {
                let pubkey = PublicKey::from_bytes(public)?;
                let sig_bytes: [u8; 64] = sig.try_into()?;
                let signature = Signature::from(sig_bytes);
                let result = pubkey
                    .verify(payload, &signature)
                    .map_err(|e| anyhow::anyhow!(e.to_string()))?;
                Ok(result)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    /*#[test]
    fn key_from_bytes_test() -> Result<()> {
        let key = Idp2pKey::Idp2pEd25519 {
            public: ed25519_dalek::PublicKey::from_bytes(&[1u8; 32])?,
        };
        let encoded = serde_json::to_string_pretty(&key)?;
        eprintln!("{encoded}");
        let decoded: Idp2pKey = serde_json::from_str(&encoded)?;
        eprintln!("{:?}", decoded);
        Ok(())
    }
    #[test]
    fn key_from_bytes_test() -> Result<()> {
        let key = Idp2pKey::Idp2pEd25519 {
            public: PublicKey::from_bytes(&[1u8; 32])?,
        };
        let bytes: Vec<u8> = key.into();
        let decoded_key = Idp2pKey::from_bytes(&bytes)?;
        matches!(decoded_key, Idp2pKey::Idp2pEd25519 { public } if public.to_bytes() == [1u8;32]  );
        Ok(())
    }
    #[test]
    fn key_serialize_test() -> Result<()> {
        let key = Idp2pKey::try_from(0xed, &[0u8; 32])?;
        let encoded = serde_json::to_string(&key).unwrap();
        let decoded: Idp2pKey = serde_json::from_str(&encoded)?;
        assert_eq!(decoded, key);
        Ok(())
    }

    #[test]
    fn agreement_key_serialize_test() -> Result<()> {
        let key = Idp2pAgreementKey::try_from(0xec, &[0u8; 32])?;
        let encoded = serde_json::to_string(&key).unwrap();
        let decoded: Idp2pAgreementKey = serde_json::from_str(&encoded)?;
        assert_eq!(decoded, key);
        Ok(())
    }

    #[test]
    fn verify_test() -> Result<()> {
        let secret = Idp2pSecret::Idp2p25519 {
            secret: EdSecret::new(),
        };
        let sig = secret.sign(&[0u8; 10]);
        let key: Idp2pKey = secret.into();
        matches!(key.verify(&[0u8; 10], &sig), Ok(true));
        Ok(())
    }*/
}

/*impl FromStr for Idp2pKey {
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
