use std::collections::{BTreeMap, BTreeSet};

use serde::{Serialize, Deserialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct IdState {
    /// Identifier
    pub id: String,

    /// Last event id
    pub event_id: String,

    /// Last event time
    pub event_timestamp: i64,

    /// Threshold
    pub threshold: u8,

    /// Next threshold
    pub next_threshold: u8,

    /// Current signers
    pub signers: BTreeMap<String, Vec<u8>>,

    /// Claims
    pub claims: BTreeMap<String, Vec<Vec<u8>>>,

    /// CID codec should be 0xed
    pub next_signers: BTreeSet<String>,

    /// All keys
    pub all_signers: BTreeSet<String>,

    /// Previous id
    pub previous_id: Option<String>,

    /// Next id
    pub next_id: Option<String>,
}
