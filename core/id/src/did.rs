use idp2p_common::{bytes::Bytes, wasmsg::Wasmsg};
use serde::{Deserialize, Serialize};
use serde_with::serde_as;

#[serde_as]
#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub struct IdProof {
    pub kid: String,
    pub timestamp: i64,
    #[serde_as(as = "Bytes")]
    pub sig: Vec<u8>,
}

#[serde_as]
#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub enum PersistedIdProof {
    CurrentKey(IdProof),
    NextKey(IdProof),
    DelegationKey {
        id: String,
        proof: Wasmsg
    },
}

#[serde_as]
#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub struct PersistedIdEvent {
    id: String,
    #[serde_as(as = "Bytes")]
    payload: Vec<u8>,
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    proofs: Vec<PersistedIdProof>,
}

#[serde_as]
#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub enum InputKind {
    #[serde(rename = "verify-inception")]
    VerifyInception(PersistedIdEvent),
    #[serde(rename = "verify-event")]
    VerifyEvent(PersistedIdEvent),
    #[serde(rename = "verify-proof")]
    VerifyProof(IdProof),
}

#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub struct PersistedId {
    id: String,
    inception: Wasmsg,
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    events: Vec<Wasmsg>,
}



/*
id
inception: Wasmsg



// write tests for idkeykind and print json of idkeykind

mod tests {
    use crate::did::IdKeyKind;

    #[test]
    fn test_idkeykind() {
        let idkeykind =
            serde_json::to_string_pretty(&IdKeyKind::DelegationKey("test".to_string())).unwrap();
        print!("{idkeykind}");
    }
}*/
