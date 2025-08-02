use super::signer::IdSigner;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub struct IdState {
    /// Identifier
    pub id: String,

    /// Last event id
    pub event_id: String,

    /// Last event time
    pub event_timestamp: DateTime<Utc>,

    /// Previous id
    pub prior_id: Option<String>,

    pub threshold: u8,
    
    pub next_threshold: u8,

    /// Signers
    pub signers: BTreeMap<String, IdSigner>,

    /// Delegates
    pub delegates: BTreeSet<String>,

    /// Current signers
    pub current_signers: BTreeSet<String>,

    /// CID codec should be 0xed
    pub next_signers: BTreeSet<String>,

    /// Claim events
    pub claim_events: BTreeMap<String, Vec<Vec<u8>>>,
}
