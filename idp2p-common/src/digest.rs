use crate::{encode_vec, ED25519_CODE, SHA256_CODE};
use anyhow::Result;
use cid::multihash::{Code, MultihashDigest};
use serde::{Deserialize, Serialize};

#[derive(PartialEq, Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum Idp2pDigest {
    Sha256 {
        #[serde(with = "encode_vec")]
        digest: Vec<u8>,
    },
}

#[derive(PartialEq, Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum Idp2pKeyDigest {
    Idp2pEd25519 { digest: Idp2pDigest },
}

impl Idp2pDigest {
    pub fn new(content: &[u8]) -> Self {
        Idp2pDigest::Sha256 {
            digest: Code::Sha2_256.digest(content).digest().to_vec(),
        }
    }
    pub fn from(code: u64, digest: &[u8]) -> Result<Self> {
        match code {
            SHA256_CODE => Ok(Idp2pDigest::Sha256 {
                digest: digest.to_vec(),
            }),
            _ => anyhow::bail!("Invalid hash code"),
        }
    }

    pub fn is_hash_of(&self, payload: &[u8]) -> bool {
        match self {
            Idp2pDigest::Sha256 { digest } => {
                let expected_digest = Code::Sha2_256.digest(payload).digest().to_vec();
                expected_digest == digest.to_owned()
            }
        }
    }
}

impl Idp2pKeyDigest {
    pub fn new(code: u64, digest: Idp2pDigest) -> Result<Self> {
        match code {
            ED25519_CODE => Ok(Self::Idp2pEd25519 { digest: digest }),
            _ => anyhow::bail!("Invalid key code"),
        }
    }

    pub fn code(&self) -> u64 {
        match self {
            Idp2pKeyDigest::Idp2pEd25519 { digest: _ } => ED25519_CODE,
        }
    }
}

#[cfg(test)]
mod tests {
    fn key_to_digest_test() {
        //let digest =
    }
}
