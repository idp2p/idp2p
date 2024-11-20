use cid::Cid;
use anyhow::Result;
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


#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct IdSnapshot {
    pub id: Cid,
    pub state: Cid,
    pub config: IdConfig,
    pub event_id: Cid,
    pub event_timestamp: String,
    pub next_signers: Vec<IdSigner>,
    pub used_signers: Vec<Cid>,
    pub mediators: Vec<String>
}

impl PersistedId {
    pub fn to_bytes(&self) -> Result<Vec<u8>> {
        todo!()
    }
    
}

impl IdSnapshot {
    pub fn to_bytes(&self) -> Result<Vec<u8>> {
        todo!()
    }
    
}