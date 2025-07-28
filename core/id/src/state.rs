use std::collections::{BTreeMap, BTreeSet};

use serde::{Deserialize, Serialize};

use crate::types::{EventRule, IdSigner};

#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub struct IdState {
    /// Identifier
    pub id: String,

    /// Previous id
    pub prior_id: Option<String>,

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
