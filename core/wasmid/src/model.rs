use crate::{
    id::DigestId,
    verification::{IdPublicKeyKind, IdSignatureKind},
};
use alloc::{collections::BTreeMap, vec::Vec};
use purewasm_codec::cbor::CborCodec;
use purewasm_core::{Codec, PureError};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PersistedIdEvent {
    pub context: DigestId, // wasm identifier
    pub command: Vec<u8>,  // IdCommand bytes
    pub event: Vec<u8>,    // IdEvent bytes
}

#[derive(Serialize, Deserialize, Debug)]
pub struct WrappedIdEvent {
    pub context: DigestId,
    pub event: Vec<u8>,
}

// Identity event, it contains wasm id and event kind
#[derive(Serialize, Deserialize, Debug)]
pub struct IdCommand {
    pub context: DigestId,
    pub body: IdCommandKind,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct IdInception {
    pub min_signer: u8,         // m of n
    pub total_signer: u8,       // total number of signers
    pub signers: Vec<DigestId>, // New signer ids
    pub sdt_state: DigestId,    // Current state of id
}

impl IdInception {
    pub fn get_id(&self) -> Result<DigestId, PureError> {
        DigestId::new_sha256(&CborCodec.to_bytes(&self)?).map_err(|e| PureError::new(&e))
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct IdMutationPayload {
    pub previous: WrappedIdEvent,                  // wasm_id
    pub min_signer: Option<u8>,                    // min number of signers(m of n)
    pub total_signer: Option<u8>,                  // total number of signers
    pub new_signers: BTreeMap<DigestId, DigestId>, // New signer ids
    pub sdt_state: Option<DigestId>,               // Current state of id
}

#[derive(Serialize, Deserialize, Debug)]
pub struct IdSignature {
    pub signer_id: DigestId,
    pub signer_pk: IdPublicKeyKind,
    pub next_signer_id: DigestId,
    pub sig_bytes: IdSignatureKind,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct IdMutation {
    pub payload: IdMutationPayload,
    pub signatures: Vec<IdSignature>,
}

impl IdMutation {
    pub fn get_id(&self) -> Result<DigestId, PureError> {
        DigestId::new_sha256(&CborCodec.to_bytes(&self)?).map_err(|e| PureError::new(&e))
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub enum IdCommandKind {
    Inception(IdInception),
    Mutation(IdMutation),
}

#[derive(Serialize, Deserialize, Debug)]
pub struct IdEvent {
    pub id: DigestId,
    pub event_id: DigestId,
    pub min_signer: u8,
    pub total_signer: u8,
    pub signers: Vec<DigestId>,
    pub sdt_state: DigestId,
}
