use idp2p_common::bytes::Bytes;
use serde::{Deserialize, Serialize};
use serde_with::serde_as;


#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub struct IdEnvelope {
    pub method: u16,
    pub major: u16,
    pub minor: u16,
    pub patch: [u8; 32],
    pub payload: Vec<u8>,
}

#[serde_as]
#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub struct PersistedIdEvent {
    pub id: String,
    #[serde_as(as = "Bytes")]
    pub payload: Vec<u8>,
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    #[serde_as(as = "Vec<Bytes>")]
    pub signatures: Vec<Vec<u8>>,
    pub proofs: Vec<IdEnvelope>
}

#[serde_as]
#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub struct PersistedId {
    pub id: String,
    pub inception: IdEnvelope,
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub events: Vec<IdEnvelope>,
}

mod tests {
    use chrono::Utc;

    use super::*;
    #[test]
    fn did_encode() {
        let did = Id {
            id: "bavetds76cgrdgsbcf7er4kvc4emfq".to_string(),
            inception: IdEventEnvelope {
                id: "bavetds76cgrdgsbcf7er4kvc4emfq".to_string(),
                version: "1.0.ba3tknc6h7n7lcw".to_string(),
                payload: vec![0x00, 0x07, 0x12, 0x15, 0x00, 0x00, 0x00, 0x00],
                proofs: vec![IdProofEnvelope {
                    id: "bavetds76cgrdgsbcf7er4kvc4emfq".to_string(),
                    purpose: "authentication".to_string(),
                    version: "1.0.ba3tknc6h7n7lcw".to_string(),
                    key_id: "badsfkjdfkdskfkld".to_string(),
                    signature: vec![0x00, 0x07, 0x12, 0x15, 0x00, 0x00, 0x00, 0x00],
                    created_at: Utc::now().timestamp(),
                }],
            },
            events: vec![],
        };

        let encoded = serde_json::to_string_pretty(&did).unwrap();
        println!("{}", encoded);
    }
}
