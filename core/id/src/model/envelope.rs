use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_with::serde_as;
use idp2p_common::bytes::Bytes;
use crate::model::state::IdState;

#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub struct IdEnvelope {
    /// idp2p:id
    pub protocol: String,
    /// 1.0
    pub version: String,
    // json value
    pub body: IdValueKind,
}

#[serde_as]
#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub struct IdProof {
    // The identity who creates proof
    pub id: String,

    // The key which signs the data
    pub key_id: String,

    // Proof purpose
    pub purpose: String,

    // Proof time
    pub created_at: DateTime<Utc>,

    // Bytes of signature
    #[serde_as(as = "Bytes")]
    pub signature: Vec<u8>,
}

#[serde_as]
#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub struct IdSignature {
    // The key which signs the data
    pub key_id: String,
    // Proof time
    pub created_at: DateTime<Utc>,
    // Bytes of signature
    #[serde_as(as = "Bytes")]
    pub bytes: Vec<u8>,
}

#[serde_as]
#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub struct IdEventEnvelope {
    pub id: String,
    pub created_at: DateTime<Utc>,
    #[serde_as(as = "Bytes")]
    pub payload: Vec<u8>,
    pub signatures: Vec<IdSignature>,
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub proofs: Vec<IdProof>
}


#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, Deserialize)]
#[serde(tag = "type", content = "value")]
pub enum IdValueKind {
    Inception(IdEventEnvelope),
    Event(IdEventEnvelope),
    Proof(IdProof),
    State(IdState),
}