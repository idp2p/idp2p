use idp2p_common::{bytes::Bytes, cbor, wasmsg::Wasmsg};
use serde::{Deserialize, Serialize};
use serde_with::serde_as;

use crate::error::IdEventError;

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
#[serde(tag = "type")]
pub enum PersistedIdProof {
    CurrentKey(IdProof),
    NextKey(IdProof),
    DelegationKey { id: String, proof: Wasmsg },
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
#[serde(tag = "type")]
pub enum IdEnvelope {
    #[serde(rename = "inception")]
    Inception(PersistedIdEvent),
    #[serde(rename = "event")]
    Event(PersistedIdEvent),
    #[serde(rename = "proof")]
    Proof(IdProof),
}

pub fn handle(input: &[u8], state: &[u8]) -> Result<Vec<u8>, IdEventError> {
    let input: IdEnvelope = cbor::decode(input)?;
    match input {
        IdEnvelope::Proof(id_proof) => todo!(),
        IdEnvelope::Inception(persisted_id_event) => todo!(),
        IdEnvelope::Event(persisted_id_event) => todo!(),
    }
}
