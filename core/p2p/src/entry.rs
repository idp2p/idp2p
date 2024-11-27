use idp2p_id::model::{id::PersistedId, view::IdView};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct IdEntry {
    pub provided: bool,
    pub view: IdView,
    pub identity: PersistedId,
    pub subscribers: Vec<String>,
}