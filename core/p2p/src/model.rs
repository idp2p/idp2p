use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PureCommand {
    pub channel: String,    // Channel identifier, it is also a db
    pub context: DigestId,  // Wasm identifier
    pub query: Vec<String>, // Key value id e.g. xxx -> id
    pub payload: Vec<u8>,   // Encoded message payload
    pub event: DigestId,    // Result identifier
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PureMessage {
    pub command: PureCommand, // Command payload
    pub previous: DigestId,   // Previous message id
    pub timestamp: Vec<u8>,   // Org timestamp
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct MessageRequest {
    pub id: DigestId, // Command payload
    pub kid: DigestId,
    pub sig: Vec<u8>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum MessageResponse {
    Command(PureCommand),
    Message(PureMessage),
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct DigestId;
