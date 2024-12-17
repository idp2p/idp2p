use cid::Cid;
use serde::{Deserialize, Serialize};

use crate::{model::PersistedId, PersistedIdEvent};


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
    Other(Vec<u8>)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum IdRequestKind {
    Message(Cid),
    Other(Vec<u8>)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum IdResponseKind {
    Message(Vec<u8>),
    Other(Vec<u8>)
}

