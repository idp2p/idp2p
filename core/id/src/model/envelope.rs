use chrono::{DateTime, Utc};
use idp2p_common::bytes::Bytes;
use serde::{Deserialize, Serialize};
use serde_with::serde_as;

use crate::error::IdError;

#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub struct IdEnvelope {
    /// idp2p:id
    pub protocol: String,
    /// 1.0
    pub version: String,
    // json value
    pub body: IdEnvelopeKind,
}

#[serde_as]
#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub struct IdProof {
    // The key which signs the data
    pub key_id: String,

    // Proof time
    pub created_at: DateTime<Utc>,

    // Bytes of signature
    #[serde_as(as = "Bytes")]
    pub signature: Vec<u8>,

    pub kind: IdProofKind
}

#[serde_as]
#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub enum IdProofKind {
    Raw,
    Envelope {
        // The identity who creates proof
        id: String,

        // Proof purpose
        purpose: String,
    },
}

#[serde_as]
#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub struct IdEventEnvelope {
    pub id: String,
    pub created_at: DateTime<Utc>,
    #[serde_as(as = "Bytes")]
    pub payload: Vec<u8>,
    pub proofs: Vec<IdProof>,
}

#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, Deserialize)]
#[serde(tag = "type", content = "value")]
pub enum IdEnvelopeKind {
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
        IdEnvelopeKind::Inception(id_event_envelope) => todo!(),
        IdEnvelopeKind::Event(id_event_envelope) => todo!(),
        IdEnvelopeKind::Proof(id_proof) => todo!(),
    }
}
