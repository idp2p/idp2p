use alloc::collections::BTreeMap;

use alloc::{string::String, vec::Vec};
use idp2p_id::state::IdState;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct IdEntry {
    pub id: String,
    pub state: IdState,
    pub inception: Vec<u8>,
    pub events: BTreeMap<String, Vec<u8>>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct IdNode {
    pub id: String,
    pub providers: Vec<String>,
}

pub struct IdClient {
    pub id: String,
    pub node_id: String,
    pub followers: Vec<String>,
    pub pending_messages: Vec<PendingMessage>,
    pub inbound_messages: Vec<InboundMessage>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PendingMessage {
    pub id: String,
    pub payload: Vec<u8>,
    pub providers: Vec<String>,
    pub timestamp: i64,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct InboundMessage {
    pub id: String,
    pub payload: Vec<u8>,
    pub timestamp: i64,
}
