use chrono::{DateTime, Utc};
use serde::{Serialize, Deserialize};

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct IdSigner {
    /// Identifier of the signer e.g. "signer"
    pub id: String,

    /// Public key bytes of the signer
    pub public_key: Vec<u8>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct IdClaim {
    /// Identifying the specific claim or attribute
    pub id: String,

    /// Binary payload containing
    pub value: Vec<u8>,

    /// When the claim is valid
    pub valid_from: Option<DateTime<Utc>>,

    /// When the claim is no longer valid
    pub valid_to: Option<DateTime<Utc>>,
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
    pub claims: Vec<IdClaim>,

    /// CID codec should be 0xed
    pub next_signers: Vec<String>,

    /// All keys
    pub all_signers: Vec<String>,

    /// Previous id
    pub previous_id: Option<String>,

    /// Next id
    pub next_id: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct IdProof {
    pub id: String,
    pub public_key: Vec<u8>,
    pub signature: Vec<u8>,
}