use alloc::{string::String, vec::Vec};
use idp2p_id::state::IdState;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub enum IdEntryKind {
    Owner,
    Client,
    Following,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum IdMessageDirection {
    From,
    To,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct IdEntry {
    pub kind: IdEntryKind,
    pub state: IdState,
    pub inception: Vec<u8>,
    pub last_event: Vec<u8>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PendingResolve {
    pub id: String,
    pub challenge: Vec<u8>,
    pub verifier: Vec<u8>,
    pub providers: Vec<String>,
    pub timestamp: u64,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PendingMessage {
    pub id: String,
    pub payload: Vec<u8>,
    pub direction: IdMessageDirection,
    pub providers: Vec<String>,
    pub timestamp: u64,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PersistedMessage {
    pub id: String,
    pub payload: Vec<u8>,
    pub timestamp: u64,
}
