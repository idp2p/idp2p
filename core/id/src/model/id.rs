use cid::Cid;
use serde::{Deserialize, Serialize};

use super::{event::PersistedIdEvent, inception::PersistedIdInception};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct PersistedId {
    pub id: Cid,
    pub incepiton: PersistedIdInception,
    pub events: Vec<PersistedIdEvent>,
}

impl PersistedId {
    pub fn to_bytes(&self) -> anyhow::Result<Vec<u8>> {
        todo!()
    }
    
}