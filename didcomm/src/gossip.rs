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

pub struct GossipMessageHandler<S: IdStore> {
    store: S,
}

impl<S: IdStore> GossipMessageHandler<S> {
    pub fn new(store: S) -> Self {
        Self { store }
    }

    pub fn handle(&self, topic: Cid, message: IdGossipMessage) {
        match message.payload {
            IdGossipMessageKind::Resolve => if store.is_provided(topic) {},
            IdGossipMessageKind::Provide(id) => {
                self.store.put(topic, id);
            }
            IdGossipMessageKind::Notify => {}
        }
    }
}
