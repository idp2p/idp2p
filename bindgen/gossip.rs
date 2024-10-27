use cid::Cid;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IdGossipMessage {
    id: Cid,
    payload: IdGossipMessageKind,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum IdGossipMessageKind {
    // Resolve identity
    Resolve,
    // Provide an identity document
    Provide { doc: Vec<u8> },
    // Notify an identity event
    NotifyEvent { event: Vec<u8> },
    // Notify message
    NotifyMessage { id: Cid },
}

pub struct GossipMessageHandler<S: IdStore> {
    store: S,
}

impl<S: IdStore> GossipMessageHandler<S> {
    pub fn new(store: S) -> Self {
        Self { store }
    }

    pub fn handle(&self, topic: Cid, message: IdGossipMessage) {
        match message.payload {
            IdGossipMessageKind::Resolve => {
                if store.is_provided(topic) {
                    // call address endpoint with id doc
                }
            }
            IdGossipMessageKind::Notify => {
                // handle event
            }
        }
    }
}
