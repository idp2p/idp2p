use std::collections::BTreeMap;

use alloc::{string::String, vec::Vec};
use idp2p_id::state::IdState;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct IdEntry {
    pub id: String,
    pub state: IdState,
    pub inception: Vec<u8>,
    pub events: BTreeMap<String, Vec<u8>>
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PeerEntry {
    pub id: String,
    pub peer_id: String,
    pub providers: Vec<String>,
    pub inbound_messages: Vec<InboundMessage>,
    pub outbound_messages: Vec<OutboundMessage>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct OutboundMessage {
    pub id: String,
    pub to: String,
    pub payload: Vec<u8>,
    pub providers: Vec<String>,
    pub timestamp: i64,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct InboundMessage {
    pub id: String,
    pub to: String,
    pub payload: Vec<u8>,
    pub timestamp: i64,
}

macro_rules! id_key {
    ($id:expr) => {
        &format!("/id/{}", $id)
    };
}
