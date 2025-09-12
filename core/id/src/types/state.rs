use idp2p_common::bytes::Bytes;
use serde::{Deserialize, Serialize};
use serde_with::serde_as;

#[serde_as]
#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub enum IdClaimValue {
    None,
    Text(String),
    Bytes(#[serde_as(as = "Bytes")] Vec<u8>),
}

/// It is useful when an identity needs claims
/// Examples:
/// Controller, Corporotional ID, Rotation Security, Mediator, Peer ...
#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub struct IdClaim {
    pub kind: String,
    pub id: String,
    pub value: IdClaimValue,
    /// Valid from timestamp.
    pub valid_from: String,
    /// Valid to timestamp.
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub valid_until: Option<String>,
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

    /// Claims
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub claims: Vec<IdClaim>,
}
