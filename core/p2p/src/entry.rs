use idp2p_id::PersistedId;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct IdEntry {
    pub digest: Vec<u8>,
    pub provided: bool,
    pub id: PersistedId,
}