use chrono::prelude::*;
use cid::Cid;
use serde::{Deserialize, Serialize};

use crate::IdConfig;

/// IdInception
///
/// The inception of the identity protocol.
///
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IdInception {
    pub config: IdConfig,
    pub state: Cid,
    pub timestamp: DateTime<Utc>,
    pub next_signers: Vec<Cid>,
    pub mediators: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum IdMediatorAction {
    Add(String),
    Remove(String)
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

