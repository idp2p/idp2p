use serde::{Deserialize, Serialize};

use crate::handler::{IdView, PersistedId};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct IdEntry {
    pub provided: bool,
    pub view: IdView,
    pub identity: PersistedId,
    pub subscribers: Vec<String>,
}
