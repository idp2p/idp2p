use cid::Cid;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IdGossipMessage {
    id: Cid,
    payload: IdGossipMessageKind
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum IdGossipMessageKind {
    // When a peer wants to resolve identity
    Resolve,
    // When a provider peer wants to send id doc(key events or service endpoint changes)
    Provide(IdentityDocument),
    // When an id wants to notify with a change or event
    Notify
}
