use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::{PersistedIdEvent, PersistedIdInception};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum IdGossipMessageKind {
    // Resolve identity
    Resolve,
    // Notify an identity event
    NotifyEvent(PersistedIdEvent),
    // Notify message(this means you have a message)
    NotifyMessage {
        id: String,
        providers: Vec<String>,
        kind: IdMessageNotifyKind 
    },
    Other(Vec<u8>),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum IdMessageNotifyKind {
    ProvideId,
    SendMessage,
    Broadcast,
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
