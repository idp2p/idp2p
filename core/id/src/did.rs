use idp2p_common::bytes::Bytes;
use serde::{Deserialize, Serialize};
use serde_with::serde_as;

#[serde_as]
#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub struct IdProof {
    pub id: String,
    pub kid: String,
    #[serde_as(as = "Bytes")]
    pub sig: Vec<u8>,
}

#[serde_as]
#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub struct PersistedIdInception {
    pub id: String,
    pub version: String,
    pub timestamp: i64,
    pub previous_id: Option<String>,
    #[serde_as(as = "Bytes")]
    pub payload: Vec<u8>,
    pub proofs: Vec<IdProof>,
}

#[serde_as]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PersistedIdEvent {
    pub id: String,
    pub version: String,
    pub previous: String,
    pub timestamp: i64,
    pub kind: String,
    #[serde_as(as = "Bytes")]
    pub payload: Vec<u8>,
    pub proofs: Vec<IdProof>,
}
