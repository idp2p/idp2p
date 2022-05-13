use anyhow::{bail, Result};
use cid::multihash::{Code, Multihash, MultihashDigest};
use serde::{de::Error, Deserialize, Serialize};

use crate::{base64url, key::Idp2pKey, Idp2pHasher};

#[derive(PartialEq, Clone)]
pub enum Idp2pKeyDigest {
    Idp2pEd25519 { mh: Multihash },
}

impl std::fmt::Debug for Idp2pKeyDigest {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Idp2pEd25519 { mh } => {
                write!(f, "Ed25519({:?})", base64url::encode_bytes(&mh.to_bytes()))
            }
        }
    }
}

impl Into<Idp2pKeyDigest> for Idp2pKey {
    fn into(self) -> Idp2pKeyDigest {
        match self {
            Idp2pKey::Idp2pEd25519 { public } => Idp2pKeyDigest::Idp2pEd25519 {
                mh: Code::Sha2_256.digest(&public.to_bytes()),
            },
        }
    }
}

impl Idp2pKeyDigest {
    pub fn try_from(code: u64, bytes: &[u8]) -> Result<Self> {
        match code {
            0xed => Ok(Idp2pKeyDigest::Idp2pEd25519 {
                mh: Multihash::from_bytes(bytes)?,
            }),
            _ => bail!(""),
        }
    }

    pub fn to_key(&self, public: &[u8]) -> Result<Idp2pKey> {
        match self {
            Idp2pKeyDigest::Idp2pEd25519 { mh } => {
                if !mh.is_hash_of(public)? {
                    bail!("Digest is not for this public")
                }
                Ok(Idp2pKey::Idp2pEd25519 {
                    public: ed25519_dalek::PublicKey::from_bytes(&public)?,
                })
            }
        }
    }
}

impl Serialize for Idp2pKeyDigest {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match self {
            Idp2pKeyDigest::Idp2pEd25519 { mh } => {
                let mut encoded: Vec<u8> = vec![0xed];
                encoded.clone_from_slice(&mh.to_bytes());
                let s = multibase::encode(multibase::Base::Base64Url, encoded);
                serializer.serialize_str(&s)
            }
        }
    }
}

impl<'de> Deserialize<'de> for Idp2pKeyDigest {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s: &str = Deserialize::deserialize(deserializer)?;
        let encoded = multibase::decode(s)
            .map_err(|err| Error::custom(err.to_string()))?
            .1;
        let mh =
            Multihash::from_bytes(&encoded[1..]).map_err(|err| Error::custom(err.to_string()))?;
        Ok(Idp2pKeyDigest::Idp2pEd25519 { mh: mh })
    }
}
