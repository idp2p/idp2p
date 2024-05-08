use alloc::{collections::BTreeMap, string::String, vec::Vec};
use purewasm_core::DigestId;
use purewasm_core::PureResult;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PureMessage {
    pub context: DigestId,                 // wasm identifier
    pub query: BTreeMap<String, DigestId>, // db/xxx -> id
    pub command: Vec<u8>,                  // Encoded message payload
    pub event: DigestId,                   // id of events, should be same with wasm execution
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PureMessageInput {
    pub context: Vec<u8>,                 // wasm raw bytes
    pub query: BTreeMap<String, Vec<u8>>, // db:xxx -> encoded query result
    pub message: PureMessage,             // Persisted message
}

#[derive(Serialize, Deserialize, Debug)]
pub struct PureEvent {
    pub context: DigestId,
    pub event: Vec<u8>, // Encoded event
}

pub fn handle(input: PureMessageInput) -> PureResult<PureEvent> {
    // each hash(input.query) == input.message.query_ids
    let result = PureEvent {
        context: input.message.context,
        event: Vec::new(),
    };
    // hash(result) == input.message.query.event_id
    Ok(result)
}
