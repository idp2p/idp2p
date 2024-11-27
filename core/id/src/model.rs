use cid::Cid;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PersistedIdProof {
    pub id: Cid,
    pub pk: Vec<u8>,
    pub sig: Vec<u8>,
}

pub mod event;
pub mod inception;
pub mod id;
pub mod view;






