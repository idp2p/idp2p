use idp2p_common::bytes::Bytes;
use serde::{Deserialize, Serialize};
use serde_with::serde_as;

#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, Deserialize)]
#[serde(tag = "type", content = "id")]
pub enum IdKeyKind {
    #[serde(rename = "current-key")]
    CurrentKey,
    #[serde(rename = "next-key")]
    NextKey,
    #[serde(rename = "delegation-key")]
    DelegationKey(String),
}

#[serde_as]
#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub struct PersistedIdProof {
    pub kind: IdKeyKind,
    pub kid: String,
    pub timestamp: i64,
    #[serde_as(as = "Bytes")]
    pub sig: Vec<u8>,
}

#[serde_as]
#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub struct PersistedIdInception {
    pub id: String,
    pub version: String,
    pub timestamp: i64,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub prior_id: Option<String>,
    #[serde_as(as = "Bytes")]
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub payload: Vec<u8>,
    pub proofs: Vec<PersistedIdProof>,
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
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub proofs: Vec<PersistedIdProof>,
}

#[serde_as]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PersistedId {
    pub id: String,
    pub inception: PersistedIdInception,
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub events: Vec<PersistedIdEvent>,
}

// write tests for idkeykind and print json of idkeykind

mod tests {
    use super::IdKeyKind;

    #[test]
    fn test_idkeykind() {
        let idkeykind = serde_json::to_string_pretty(&IdKeyKind::CurrentKey).unwrap();
        print!("{idkeykind}");
    }
}