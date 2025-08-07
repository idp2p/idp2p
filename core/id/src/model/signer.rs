use chrono::{DateTime, Utc};
use idp2p_common::bytes::Bytes;
use serde::{Deserialize, Serialize};
use serde_with::serde_as;

/// Signer
///
/// Represents a signer of an identifier.
#[serde_as]
#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub struct IdSigner {
    /// Public key of the signer.
    #[serde_as(as = "Bytes")]
    pub public_key: Vec<u8>,
    /// Valid from timestamp.
    pub valid_from: DateTime<Utc>,
    /// Valid to timestamp.
    pub valid_to: Option<DateTime<Utc>>,
}

#[serde_as]
#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub struct IdProof {
    // The key which signs the data
    pub key_id: String,
    // Proof time
    pub created_at: DateTime<Utc>,
    // Bytes of signature
    #[serde_as(as = "Bytes")]
    pub bytes: Vec<u8>,
}



impl IdSigner {
    pub fn new(public_key: &[u8]) -> Self {
        Self {
            public_key: public_key.to_vec(),
            valid_from: chrono::Utc::now(),
            valid_to: None,
        }
    }
    pub fn is_valid(&self, now: DateTime<Utc>) -> bool {
        self.valid_from <= now && (self.valid_to.is_none() || self.valid_to.unwrap() >= now)
    }
}
