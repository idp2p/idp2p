use alloc::collections::BTreeMap;

use chrono::{DateTime, Utc};
use idp2p_common::bytes::Bytes;
use serde::{Deserialize, Serialize};
use serde_with::serde_as;

use crate::error::IdError;
use crate::model::{event, inception};
use crate::{VALID_FROM, VERSION};
use idp2p_common::cbor;

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
pub struct IdProofEnvelope {
    // The identity who creates proof
    pub id: String,

    // Proof purpose
    pub purpose: String,

    // The key which signs the data
    pub key_id: String,

    // Proof time
    pub created_at: DateTime<Utc>,

    // Bytes of signature
    #[serde_as(as = "Bytes")]
    pub signature: Vec<u8>,
}

#[serde_as]
#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub struct IdEventEnvelope {
    pub id: String,
    pub created_at: DateTime<Utc>,
    #[serde_as(as = "Bytes")]
    pub payload: Vec<u8>,
    pub proofs: BTreeMap<String, Vec<u8>>,
    pub delegator_proofs: Vec<IdProofEnvelope>,
}

#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, Deserialize)]
#[serde(tag = "type", content = "value")]
pub enum IdEnvelopeKind {
    Inception(IdEventEnvelope),
    Event(IdEventEnvelope),
    Proof(IdProofEnvelope),
}

impl TryFrom<&Vec<u8>> for IdEnvelope {
    type Error = IdError;

    fn try_from(value: &Vec<u8>) -> Result<Self, Self::Error> {
        Ok(serde_json::from_slice(value)?)
    }
}

pub fn handle_message(message: Vec<u8>, state: Option<Vec<u8>>) -> Result<Vec<u8>, IdError> {
    let en: IdEnvelope = serde_json::from_slice(&message)?;
    // Basic envelope checks
    if en.protocol != "idp2p:id" {
        return Err(IdError::Other("unsupported protocol".into()));
    }
    if en.version != VERSION {
        return Err(IdError::Other("unsupported version".into()));
    }
    // Basic protocol routing; additional validation (CID/hash) happens in verifiers.
    match en.body {
        IdEnvelopeKind::Inception(id_event_envelope) => {
            let out = inception::verify(&id_event_envelope).map_err(IdError::from)?;
            Ok(out)
        }
        IdEnvelopeKind::Event(id_event_envelope) => {
            // Expect prior state to be provided; decode and pass by mutable ref.
            let mut id_state = match state {
                Some(bytes) => cbor::decode(&bytes).map_err(IdError::from)?,
                None => {
                    // No state provided; treat as JSON error domain-wise for simplicity.
                    return Err(IdError::Other("missing state for event".into()));
                }
            };
            let out = event::verify(&id_event_envelope, &mut id_state).map_err(IdError::from)?;
            Ok(out)
        }
        IdEnvelopeKind::Proof(id_proof) => {
            // Minimal validation only; actual signature verification depends on caller context.
            if id_proof.id.is_empty() || id_proof.key_id.is_empty() {
                return Err(IdError::Other("invalid proof: missing ids".into()));
            }
            if id_proof.signature.is_empty() {
                return Err(IdError::Other("invalid proof: empty signature".into()));
            }
            let valid_from: DateTime<Utc> = VALID_FROM.parse().expect("Invalid date format");
            if id_proof.created_at < valid_from {
                return Err(IdError::Other("invalid proof: timestamp".into()));
            }
            Ok(Vec::new())
        }
    }
}
