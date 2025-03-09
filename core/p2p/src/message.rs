use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::{PersistedIdEvent, PersistedIdInception};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum IdMessageKind {
    Direct,
    Broadcast,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum IdGossipMessageKind {
    // Resolve identity
    Resolve,
    // Provide identity, takes a list of providers
    Provide(Vec<String>),
    // Notify an identity event
    NotifyEvent(PersistedIdEvent),
    // Notify message(this means you have a message)
    NotifyMessage {
        id: String,
        providers: Vec<String>,
        kind: IdMessageKind 
    },
    Other(Vec<u8>),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IdGossipMessage{
    pub id: String,
    pub payload: IdGossipMessageKind
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum IdMessageHandlerRequestKind {
    MessageRequest {
        id: String,
        message_id: String
    },
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
