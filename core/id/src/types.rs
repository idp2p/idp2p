use serde::{Deserialize, Serialize};
use serde_with::serde_as;
use idp2p_common::bytes::Bytes;

#[serde_as]
#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub struct IdProof {
    pub id: String,
    pub kid: String,
    #[serde_as(as = "Bytes")]
    pub sig: Vec<u8>,
}

#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub struct IdSigner {
    pub id: String,
    pub pk: Vec<u8>,
}

#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub struct IdClaim {
    pub key: String,
    pub value: Vec<Vec<u8>>,
}

#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub struct PersistedIdInception {
    pub id: String,
    pub version: String,
    pub payload: Vec<u8>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PersistedIdEvent {
    pub id: String,
    pub version: String,
    pub payload: Vec<u8>,
    pub proofs: Vec<IdProof>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EventRuleIdKind {
    Current,
    Other(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventRuleExpr {
    pub id: EventRuleIdKind,
    pub threshold: u8,
}

pub type EventRule = Vec<Vec<EventRuleExpr>>;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IdConfig {
    pub rotation_rule: EventRule,
    pub interaction_rule: EventRule,
    pub revocation_rule: EventRule,
    pub migration_rule: EventRule
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct IdState {
    /// Identifier
    pub id: String,

    /// Previous id
    pub previous_id: Option<String>,
    
    /// Id Config
    pub config: IdConfig,

    /// Last event id
    pub event_id: String,

    /// Last event time
    pub event_timestamp: i64,

    /// Current signers
    pub signers: Vec<IdSigner>,

    /// Claims
    pub claims: Vec<IdClaim>,

    /// CID codec should be 0xed
    pub next_signers: Vec<String>,

    /// All keys
    pub all_signers: Vec<String>,
}
