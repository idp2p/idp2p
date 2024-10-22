use cid::Cid;
use serde::{Deserialize, Serialize};

use crate::{action::VerificationMethod, config::IdConfig, event::PersistedIdEvent, inception::PersistedIdInception, signer::IdSigner};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct IdSnapshot {
    pub id: Vec<u8>,
    pub config: IdConfig,
    pub event_id: Vec<u8>,
    pub event_timestamp: String,
    pub next_signers: Vec<IdSigner>,
    pub authentication: Vec<VerificationMethod>,
    pub assertion_method: Vec<VerificationMethod>,
    pub key_agreement: Vec<VerificationMethod>,
    pub mediators: Vec<Cid>,    
    pub used_signers: Vec<Cid>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct PersistedIdDocument {
    pub id: Cid,
    pub incepiton: PersistedIdInception,
    pub events: Vec<PersistedIdEvent>,
}