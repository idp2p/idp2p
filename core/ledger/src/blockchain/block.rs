// channel: xxx
// schema: /wasmid/blockchain/bac....
// payload: messages, timestamp
// projections: last_block, channel
// result_proof: hash(last_block = new, ...)

use serde::{Deserialize, Serialize};

use crate::{message::PureMessage, DigestId};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct BlockPayload {
    pub channel: String,            // Schema identifier, it is also a db id
    pub messages: Vec<PureMessage>, // Message payload
    pub previous: DigestId,         // Previous message id
    pub timestamp: u64,             // Timestamp
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct BlockSignature {
    pub signer_id: DigestId,
    pub signer_kid: DigestId,
    pub block_id: DigestId, // Block payload
    pub signature: Vec<u8>, // Org timestamp, signature of
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Block {
    id: DigestId,          // Message identifier
    payload: BlockPayload, //degf
    signature: BlockSignature,
}
