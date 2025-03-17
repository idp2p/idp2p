use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub enum IdEntryKind {
    Owner,
    Client,
    Following,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct IdEntry {
    pub kind: IdEntryKind,
    pub state: IdState,
    pub inception: PersistedIdInception,
    pub events: HashMap<String, PersistedIdEvent>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum IdNetworkEvent {
    Pubsub(IdPubsubMessageKind),
    Request(IdRequestKind),
    Response(IdResponseKind),
}



#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum IdRequestKind {
    MessageRequest {
        id: String,
        message_id: String
    },
    IdRequest(String)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum IdResponseKind {
    MessageResponse(Vec<u8>),
    IdResponse {
        inception: PersistedIdInception,
        events: HashMap<String, PersistedIdEvent>,
    }
}

