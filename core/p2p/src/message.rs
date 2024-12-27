use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::{PersistedIdEvent, PersistedIdInception};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum IdGossipMessageKind {
    // Resolve identity
    Resolve,
    // Notify an identity event
    NotifyEvent {
        version: String,
        event: PersistedIdEvent,
    },
    // Notify message(this means you have a message)
    NotifyMessage {
        id: String,
        providers: Vec<String>,
        direction: IdGossipMessageDirection 
    },
    Other(Vec<u8>),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum IdGossipMessageDirection {
    From,
    To
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum IdMessageHandlerRequestKind {
    MessageRequest(String),
    IdRequest(String)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum IdMessageHandlerResponseKind {
    MessageResponse(Vec<u8>),
    IdResponse {
        inception: PersistedIdInception,
        events: HashMap<String, PersistedIdEvent>,
    }
}
