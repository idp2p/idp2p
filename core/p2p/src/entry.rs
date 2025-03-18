use alloc::collections::BTreeMap;

use idp2p_id::{event::PersistedIdEvent, inception::PersistedIdInception, types::IdState};
use serde::{Deserialize, Serialize};

use crate::types::IdEntryKind;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct IdEntry {
    pub kind: IdEntryKind,
    pub state: IdState,
    pub inception: PersistedIdInception,
    pub events: BTreeMap<String, PersistedIdEvent>,
}