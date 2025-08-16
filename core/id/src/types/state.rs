use serde::{Deserialize, Serialize};
use serde_with::serde_as;
use idp2p_common::bytes::Bytes;

use crate::IdSigner;

#[serde_as]
#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub struct IdDelegator {
    pub id: String,
    
    pub restrictions: Vec<String>,
}

#[serde_as]
#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub struct IdClaimEvent {
    pub id: String,
    pub created_at: String,
    #[serde_as(as = "Bytes")]
    pub payload: Vec<u8>,
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
    pub prior_id: Option<String>,

    // Current threshold
    pub threshold: u8,
    
    // Next threshold
    pub next_threshold: u8,

    /// Signers
    pub signers: Vec<IdSigner>,

    /// Delegators
    pub delegators: Vec<IdDelegator>,

    /// Current signers
    pub current_signers: Vec<String>,

    /// CID codec should be 0xed
    pub next_signers: Vec<String>,

    /// Claim events
    pub claim_events: Vec<IdClaimEvent>,
}