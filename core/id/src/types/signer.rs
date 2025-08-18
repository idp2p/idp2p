use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_with::serde_as;
use idp2p_common::bytes::Bytes;

/// Signer
///
/// Represents a signer of an identifier.
#[serde_as]
#[derive(Debug, Clone, Hash, Serialize, Deserialize)]
pub struct IdSigner {
    pub id: String,
    /// Public key of the signer.
    #[serde_as(as = "Bytes")]
    pub public_key: Vec<u8>,
    /// Valid from timestamp.
    pub valid_from: String,
    /// Valid to timestamp.
    pub valid_until: Option<String>,
}

impl IdSigner {
    pub fn new(id: &str,public_key: &[u8]) -> Self {
        Self {
            id: id.into(),
            public_key: public_key.to_vec(),
            valid_from: Utc::now().to_string(),
            valid_until: None,
        }
    }
    /*pub fn is_valid(&self, now: DateTime<Utc>) -> bool {
        let valid_from = self.valid_from.parse().map_err(op)
        self.valid_from <= now && (self.valid_until.is_none() || self.valid_until.unwrap() >= now)
    }*/
}

impl Eq for IdSigner {}
impl PartialEq for IdSigner {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl Ord for IdSigner {
    fn cmp(&self, other: &Self) -> core::cmp::Ordering {
        self.id.cmp(&other.id)
    }
}

impl PartialOrd for IdSigner {
    fn partial_cmp(&self, other: &Self) -> Option<core::cmp::Ordering> {
        Some(self.cmp(other))
    }
}