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
pub enum IdMessageHandlerRequestKind {
    MessageRequest(Cid),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum IdMessageHandlerResponseKind {
    MessageResponse(Vec<u8>),
}

