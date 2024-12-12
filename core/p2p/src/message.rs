use cid::Cid;
use serde::{Deserialize, Serialize};

use crate::{model::PersistedId, PersistedIdEvent};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum IdMessageRequest {
    Get(String),
    Provide(Vec<u8>),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum IdGossipMessageKind {
    // Resolve identity
    Resolve,
    // Provide an identity document
    Provide {
        id: PersistedId,
    },
    // Notify an identity event
    NotifyEvent {
        version: u64,
        event: PersistedIdEvent,
    },
    // Notify message(this means you have a message)
    NotifyMessage{
        id: Cid,
        providers: Vec<String>,
    },
}
