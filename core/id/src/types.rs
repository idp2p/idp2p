use idp2p_common::bytes::Bytes;
use serde::{Deserialize, Serialize};
use serde_with::serde_as;

#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub enum IdKeyType {
    #[serde(rename = "current-key")]
    CurrentKey,
    #[serde(rename = "next-key")]
    NextKey,
    #[serde(rename = "delegation-key")]
    DelegationKey,
}

#[serde_as]
#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub struct PersistedIdProof {
    pub id: String,
    pub version: String,
    pub key_type: IdKeyType,
    pub key_id: String,
    #[serde_as(as = "Bytes")]
    pub signature: Vec<u8>,
}

#[serde_as]
#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub struct PersistedIdEvent {
    pub id: String,
    pub version: String,
    #[serde_as(as = "Bytes")]
    pub payload: Vec<u8>,
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub proofs: Vec<PersistedIdProof>,
}

#[serde_as]
#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub struct PersistedId {
    pub id: String,
    pub inception: PersistedIdEvent,
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub events: Vec<PersistedIdEvent>,
}

mod tests {
    use crate::types::*;

    #[test]
    fn did_encode() {
        let did = PersistedId {
            id: "bavetds76cgrdgsbcf7er4kvc4emfq".to_string(),
            inception: PersistedIdEvent {
                id: "bavetds76cgrdgsbcf7er4kvc4emfq".to_string(),
                version: "ba3tknc6h7n7lcw".to_string(),
                payload: vec![0x00, 0x07, 0x12, 0x15, 0x00, 0x00, 0x00, 0x00],
                proofs: vec![PersistedIdProof {
                    id: "bavetds76cgrdgsbcf7er4kvc4emfq".to_string(),
                    version: "ba3tknc6h7n7lcw".to_string(),
                    key_type: IdKeyType::CurrentKey,
                    key_id: "badsfkjdfkdskfkld".to_string(),
                    signature: vec![0x00, 0x07, 0x12, 0x15, 0x00, 0x00, 0x00, 0x00],
                }],
            },
            events: vec![],
        };

        let encoded = serde_json::to_string_pretty(&did).unwrap();
        println!("{}", encoded);
    }
}
