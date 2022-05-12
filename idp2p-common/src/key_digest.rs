use anyhow::{bail, Result};
use cid::multihash::{Multihash, Code, MultihashDigest};

use crate::{key::Idp2pKey, Idp2pHasher, base64url};

#[derive(PartialEq, Clone)]
pub enum Idp2pKeyDigest {
    Idp2pEd25519 { mh: Multihash },
}

impl std::fmt::Debug for Idp2pKeyDigest {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Idp2pEd25519 { mh } => write!(
                f,
                "Ed25519({:?})",
                base64url::encode_bytes(&mh.to_bytes())
            ),
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
