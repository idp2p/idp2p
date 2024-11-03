use idp2p_id::{IdSnapshot, PersistedId};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct IdEntry {
    pub provided: bool,
    pub snapshot: IdSnapshot,
    pub identity: PersistedId,
    pub subscribers: Vec<String>,
}