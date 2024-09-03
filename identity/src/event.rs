use cid::Cid;
use serde::{Deserialize, Serialize};

const VERSION: &'static str = "id@1.0.0"; 

#[derive(Debug, Serialize, Deserialize)]
pub struct IdInception {
    pub version: String,
    pub next_signer_id: Cid,
    pub next_recovery_id: Cid,
    pub timestamp: i64,
    pub state_root: Cid,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct IdEvent {
    pub version: String,
    pub timestamp: i64,
    pub previous: Cid,
    pub payload: IdEventKind,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "kind")]
pub enum IdEventKind {
    Mutation {
        signer_pk: Vec<u8>,
        next_signer_id: Cid,
        state_root: Cid,
    },
    Recovery {
        recovery_pk: Vec<u8>,
        next_signer_id: Cid,
        next_recovery_id: Cid,
    },
}

