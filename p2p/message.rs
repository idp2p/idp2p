use cid::Cid;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IdGossipMessage {
    id: Cid,
    payload: IdGossipMessageKind,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum IdGossipMessageKind {
    // When a peer wants to resolve identity
    Resolve { address: String },
    // When an id wants to notify with a change or event
    Notify { address: String, event_id: Cid },
}