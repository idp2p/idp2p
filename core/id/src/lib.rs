use cid::Cid;
use anyhow::Result;
use chrono::{DateTime, Utc};
use config::IdConfig;
use event::PersistedIdEvent;
use inception::PersistedIdInception;
use serde::{Deserialize, Serialize};
use signer::IdSigner;

pub mod action;
pub mod config;
pub mod event;
pub mod inception;
pub mod signer;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PersistedIdProof {
    pub id: Cid,
    pub pk: Vec<u8>,
    pub sig: Vec<u8>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct PersistedId {
    pub id: Cid,
    pub incepiton: PersistedIdInception,
    pub events: Vec<PersistedIdEvent>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerificationMethod {
    pub id: Vec<u8>,
    pub pk: Vec<u8>,
    pub valid_from: DateTime<Utc>,
    pub valid_until: DateTime<Utc>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct IdSnapshot {
    pub id: Vec<u8>,
    pub config: IdConfig,
    pub event_id: Vec<u8>,
    pub event_timestamp: String,
    pub next_signers: Vec<IdSigner>,
    pub used_signers: Vec<Cid>,
    pub mediators: Vec<Cid>,    
    pub authentication: Vec<VerificationMethod>,
    pub assertion_method: Vec<VerificationMethod>,
    pub key_agreement: Vec<VerificationMethod>,
}

impl IdSnapshot {
    pub fn to_bytes(&self) -> Result<Vec<u8>> {
        todo!()
    }
    
}

impl VerificationMethod {
    pub fn validate(&self) -> Result<()> {
       todo!()
    }
}