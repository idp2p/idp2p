use serde::{Deserialize, Serialize};

use crate::DigestId;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PureMessage {
    pub channel: String,          // Schema identifier, it is also a db id
    pub schema: String,           // Contract id and version <id>://<multibase wasm id>
    pub payload: Vec<u8>,         // Encoded message payload
    pub projections: Vec<String>, // Query keys(e.g /channel, /users/id, /users_count, /custom)
    pub result_proof: DigestId,   // Result identifier
}

