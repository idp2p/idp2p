use std::collections::{BTreeMap, BTreeSet};

use serde::{Deserialize, Serialize};
use serde_with::serde_as;
use idp2p_common::bytes::Bytes;

use crate::did::IdKeyKind;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventRuleItem {
    pub kind: IdKeyKind,
    pub threshold: u8,
}

pub type EventRule = Vec<Vec<EventRuleItem>>;

#[serde_as]
#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub struct IdSigner {
    #[serde_as(as = "Bytes")]
    pub public_key: Vec<u8>,
    pub valid_from: i64,
    pub valid_to: Option<i64>,
}


impl IdSigner {
    pub fn new(public_key: &[u8]) -> Self {
        Self {
            public_key: public_key.to_vec(),
            valid_from: chrono::Utc::now().timestamp(),
            valid_to: None
        }
    }
    pub fn is_valid(&self, now: i64) -> bool {
        self.valid_from <= now && (self.valid_to.is_none() || self.valid_to.unwrap() >= now)
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct IdState {
    /// Identifier
    pub id: String,

    /// Previous id
    pub previous_id: Option<String>,
    
    /// Rotation rule
    pub rotation_rule: EventRule,

    /// Interaction rule
    pub interaction_rule: EventRule,

    /// Revocation rule
    pub revocation_rule: EventRule,

    /// Migration rule
    pub migration_rule: EventRule,

    /// Last event id
    pub event_id: String,

    /// Last event time
    pub event_timestamp: i64,

    /// Signers
    pub signers: BTreeMap<String, IdSigner>,

    /// Current signers
    pub current_signers: BTreeSet<String>,

    /// CID codec should be 0xed
    pub next_signers: BTreeSet<String>,

    /// Claim events
    pub claim_events: BTreeMap<String, Vec<Vec<u8>>>, 
}


