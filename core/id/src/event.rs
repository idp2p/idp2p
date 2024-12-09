use chrono::{DateTime, Utc};
use cid::Cid;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum IdActionKind {
    AddMediator(Cid),
    RemoveMediator(Cid),
    UpdateState(Cid),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum IdEventPayload {
    Action(Vec<IdActionKind>),
    CancelEvent(Cid),
    UpgradeId(Cid),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IdEvent {
    pub timestamp: DateTime<Utc>,
    pub previous: Cid,
    pub signers: Vec<Cid>,
    pub payload: IdEventPayload,
    pub next_signers: Vec<Cid>,
}


