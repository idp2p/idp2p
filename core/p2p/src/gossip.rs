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
    Provide { doc: IdDocument },
    // Notify an identity event
    NotifyEvent { event: IdEvent },
    // Notify message
    NotifyMessage { id: Cid },
}

#[derive(NetworkBehaviour)]
#[behaviour(to_swarm = "Idp2pNodeEvent")]
pub struct IdGossipBehaviour {
    pub id_mdns: MdnsBehaviour,
    pub id_gossipsub: GossipsubBehaviour,
    pub id_resolve: CborBehaviour<IdDocument, ()>,
    pub id_message: CborBehaviour<IdDirectMessage, ()>,
    pub id_request: CborBehaviour<IdRequest, ()>,
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

pub struct ResolveHandler<S: IdStore> {
    store: S,
}

impl<S: IdStore> ResolveHandler<S> {
    pub fn new(store: S) -> Self {
        Self { store }
    }

    pub fn handle(&self) {}
}
