use cid::Cid;
use serde::{Deserialize, Serialize};

use super::PersistedIdProof;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PersistedIdEvent {
    pub id: Cid,
    pub payload: Vec<u8>,
    pub proofs: Vec<PersistedIdProof>,
}