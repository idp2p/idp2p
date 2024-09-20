// channel: xxx
// schema: /wasmid/blockchain/bac....
// payload: messages, timestamp
// projections: last_block, channel
// result_proof: hash(last_block = new, ...)

use serde::{Deserialize, Serialize};

use crate::{message::PureMessage, DigestId};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PureMessage {
    pub channel: String,          // Db
    pub wasmid: String,           // Contract id and version /wasmid/<id>/<multibase id>
    pub payload: Vec<u8>,         // Encoded message payload
    pub projections: Vec<String>, // Keys should be queried
    pub next_state: DigestId,     // Result identifier
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct BlockPayload {
    pub channel: String,            // Db identifier
    pub messages: Vec<PureMessage>, // Message payload
    pub previous: DigestId,         // Previous message id
    pub timestamp: u64,             // Timestamp
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct BlockSignature {
    pub signer_id: DigestId,
    pub signer_kid: DigestId,
    pub signer_key: Vec<u8>,
    pub block_id: DigestId, // Block payload
    pub signature: Vec<u8>, // Org timestamp, signature of
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Block {
    id: DigestId,          // Message identifier
    payload: BlockPayload, //degf
    signature: BlockSignature,
}
