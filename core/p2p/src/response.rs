use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum IdResponseKind {
    MessageResponse(Vec<u8>),
    IdResponse {
        inception: Vec<u8>,
        events: BTreeMap<String, Vec<u8>>,
    }
}