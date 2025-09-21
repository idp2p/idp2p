use idp2p_common::bytes::Bytes;
use serde::{Deserialize, Serialize};
use serde_with::serde_as;

use crate::internal::{
    claim::{IdClaimCreateEvent, IdClaimRevokeEvent},
    error::IdEventError,
};

#[serde_as]
#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub struct IdClaimValue {
    pub id: String,
    /// Valid from timestamp.
    pub valid_from: String,
    /// Valid to timestamp.
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub valid_until: Option<String>,
    pub payload: Option<Vec<u8>>,
}

/// It is useful when an identity needs claims
/// Examples:
/// Controller, Corporotional ID, Rotation Security, Mediator, Peer ...
#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub struct IdClaim {
    pub key: String,
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub values: Vec<IdClaimValue>,
}

#[serde_as]
#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub struct IdSigner {
    pub id: String,
    /// Public key of the signer.
    #[serde_as(as = "Bytes")]
    pub public_key: Vec<u8>,
    /// Valid from timestamp.
    pub valid_from: String,
    /// Valid to timestamp.
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub valid_until: Option<String>,
}

#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub struct IdState {
    /// Identifier
    pub id: String,

    /// Last event id
    pub event_id: String,

    /// Last event time
    pub event_timestamp: String,

    /// Previous id
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub prior_id: Option<String>,

    /// Next id cid
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub next_id: Option<String>,

    // Current threshold
    pub threshold: u8,

    // Next threshold
    pub next_threshold: u8,

    /// Signers
    pub signers: Vec<IdSigner>,

    /// Current signers
    pub current_signers: Vec<String>,

    /// CID codec should be 0xed
    pub next_signers: Vec<String>,

    /// Delegators
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub delegators: Vec<String>,

    /// Claims
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub claims: Vec<IdClaim>,

    pub revoked: bool,

    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub revoked_at: Option<String>,
}

impl IdState {
    pub fn add_claim(&mut self, event: IdClaimCreateEvent, valid_from: &str) {
        // Find existing claim with the given key
        if let Some(claim) = self.claims.iter_mut().find(|c| c.key == event.key) {
            // Check if a value with this id already exists
            if !claim.values.iter().any(|v| v.id == event.id) {
                // Add new value only if id doesn't exist
                claim.values.push(event.to_state(valid_from));
            }
        } else {
            // Create new claim with the value
            self.claims.push(IdClaim {
                key: event.key.to_string(),
                values: vec![event.to_state(valid_from)],
            });
        }
    }

    pub fn revoke_claim(
        &mut self,
        event: IdClaimRevokeEvent,
        valid_until: &str,
    ) -> Result<(), IdEventError> {
        // Find existing claim with the given key
        if let Some(claim) = self.claims.iter_mut().find(|c| c.key == event.key) {
            // Find existing value with the given id
            if let Some(value) = claim.values.iter_mut().find(|v| v.id == event.id) {
                value.valid_until = Some(valid_until.to_string());
                return Ok(());
            }
        }
        Err(IdEventError::InvalidClaim(event.key.to_string()))
    }
}
