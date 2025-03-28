use serde::{Serialize, Deserialize};

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct IdSigner {
    /// Identifier of the signer e.g. "signer"
    pub id: String,

    /// Public key bytes of the signer
    pub public_key: Vec<u8>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct IdClaimEvent {
    /// Identifying the specific claim or attribute
    pub id: String,

    /// Binary payload `Wasmsg`
    pub payload: Vec<u8>,
}

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
    pub signers: Vec<IdSigner>,

    /// Claims
    pub claims: Vec<IdClaimEvent>,

    /// CID codec should be 0xed
    pub next_signers: Vec<String>,

    /// All keys
    pub all_signers: Vec<String>,

    /// Previous id
    pub previous_id: Option<String>,

    /// Next id
    pub next_id: Option<String>,
}
