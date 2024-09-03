use cid::Cid;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum IdGossipMessage {
    // When a peer wants to resolve identity
    Resolve,
    // When a peer wants to provide identity
    Provide,
    // When a peer wants to withdraw identity
    Withdraw,
    // When a id wants to notify with a did change
    Notify(Cid),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum IdRequest {
    // When a peer wants to respond to subscription
    SubscriptionResponse(Cid),
    // When an identity wants to send a gossip message
    GossipMessage(IdGossipMessage),
    // When an identity wants to send a message to another identity
    IdMessage { id: Cid, body: Vec<u8> },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum IdResponse {
   Ok
}

pub struct IdStore {
    pub provided_ids: HashMap<Cid, Cid>,
    pub subscribed_ids: HashMap<Cid, Vec<Cid>>,
}