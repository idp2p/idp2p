use serde::{Deserialize, Serialize};
use serde_with::serde_as;
use idp2p_common::bytes::Bytes;

/// It is useful when an identity needs to delegate some functions
/// Examples:
/// Controller, Corporotional ID, Rotation Security, Mediator, Peer ... 
#[serde_as]
#[derive(Debug, Clone, Hash, Serialize, Deserialize)]
pub struct IdDelegator {
    pub id: String,    
    pub scope: Vec<String>,
    /// Valid from timestamp.
    pub valid_from: String,
    /// Valid to timestamp.
    pub valid_until: Option<String>,
}

#[serde_as]
#[derive(Debug, Clone, Hash, Serialize, Deserialize)]
pub struct IdClaim {
    pub r#type: String,
    pub id: String,
    #[serde_as(as = "Bytes")]
    pub payload: Vec<u8>,
    /// Valid from timestamp.
    pub valid_from: String,
    /// Valid to timestamp.
    pub valid_until: Option<String>,
}

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

#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub struct IdState {
    /// Identifier
    pub id: String,

    /// Last event id
    pub event_id: String,

    /// Last event time
    pub event_timestamp: String,

    /// Also known as
    pub aka: Vec<String>,

    /// Previous id
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

     /// Delegators
    pub delegators: Vec<IdDelegator>,

    /// Claims
    pub claims: Vec<IdClaim>,
}
