use serde::{Deserialize, Serialize};
use serde_with::serde_as;
use idp2p_common::bytes::Bytes;

use crate::IdSigner;

#[serde_as]
#[derive(Debug, Clone, Hash, Serialize, Deserialize)]
pub struct IdDelegator {
    pub id: String,    
    pub restrictions: Vec<String>,
}

#[serde_as]
#[derive(Debug, Clone, Hash, Serialize, Deserialize)]
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

    /// Current signers
    pub current_signers: Vec<String>,

    /// CID codec should be 0xed
    pub next_signers: Vec<String>,

     /// Delegators
    pub delegators: Vec<IdDelegator>,

     /// Delegators
    pub providers: Vec<String>,

    /// Claim events
    pub claim_events: Vec<IdClaimEvent>,
}


impl Eq for IdDelegator {}

impl PartialEq for IdDelegator {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl Ord for IdDelegator {
    fn cmp(&self, other: &Self) -> core::cmp::Ordering {
        self.id.cmp(&other.id)
    }
}

impl PartialOrd for IdDelegator {
    fn partial_cmp(&self, other: &Self) -> Option<core::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Eq for IdClaimEvent {}

impl PartialEq for IdClaimEvent {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl Ord for IdClaimEvent {
    fn cmp(&self, other: &Self) -> core::cmp::Ordering {
        self.id.cmp(&other.id)
    }
}

impl PartialOrd for IdClaimEvent {
    fn partial_cmp(&self, other: &Self) -> Option<core::cmp::Ordering> {
        Some(self.cmp(other))
    }
}