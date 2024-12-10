use chrono::{DateTime, Utc};
use cid::Cid;
use serde::{Deserialize, Serialize};

use crate::IdConfig;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum IdMediatorAction {
    Add(Cid),
    Remove(Cid)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IdAction {
    pub state: Option<Cid>,
    pub mediators: Vec<IdMediatorAction>
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum IdEventPayload {
    Action(IdAction),
    Recovery(Option<IdConfig>),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IdEvent {
    pub timestamp: DateTime<Utc>,
    pub previous: Cid,
    pub signers: Vec<Cid>,
    pub payload: IdEventPayload,
    pub next_signers: Vec<Cid>,
}


