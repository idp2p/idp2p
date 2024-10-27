use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub struct IdEntry {
    pub digest: Vec<u8>,
    pub provided: bool,
    pub document: Vec<u8>,
}