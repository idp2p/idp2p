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
        version: String,
        event: PersistedIdEvent,
    },
    // Notify message(this means you have a message)
    NotifyMessage {
        id: String,
        providers: Vec<String>,
    },
    Other(Vec<u8>),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum IdMessageHandlerRequestKind {
    MessageRequest(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum IdMessageHandlerResponseKind {
    MessageResponse(Vec<u8>),
}
