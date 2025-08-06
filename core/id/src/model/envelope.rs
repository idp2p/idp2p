use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_with::serde_as;
use idp2p_common::bytes::Bytes;

use crate::error::IdError;

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
}

impl TryFrom<&Vec<u8>> for IdEnvelope {
    type Error = IdError;

    fn try_from(value: &Vec<u8>) -> Result<Self, Self::Error> {
        todo!()
    }
}

pub fn handle_message(message: Vec<u8>, state: Option<Vec<u8>>) -> Result<Vec<u8>, IdError> {
    let en: IdEnvelope = serde_json::from_slice(&message)?;
    match en.body {
        IdValueKind::Inception(id_event_envelope) => todo!(),
        IdValueKind::Event(id_event_envelope) => todo!(),
        IdValueKind::Proof(id_proof) => todo!(),
    }
    todo!()
}