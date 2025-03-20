use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub enum IdEntryKind {
    Owner,
    Client,
    Following,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum IdMessageDirection {
    From,
    To,
}

/* 

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
}*/

