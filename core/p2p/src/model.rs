use idp2p_id::types::IdState;
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


#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct IdEntry {
    pub kind: IdEntryKind,
    pub state: IdState,
    pub inception: Vec<u8>,
    pub last_event: Vec<u8>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct IdMessage {
    pub id: String,
    pub from: String,
    pub to: Vec<String>,  // If empty for all followers
    pub payload: Vec<u8>,
}