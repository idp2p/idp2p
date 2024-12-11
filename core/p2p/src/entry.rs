use std::collections::HashMap;

use cid::Cid;
use serde::{Deserialize, Serialize};

use crate::{IdView, PersistedIdEvent, PersistedIdInception};

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
pub struct PersistedId {
    // Identity id
    pub id: Vec<u8>,
    // Specifies the version of inception
    pub version: u64,
    // Inception id and payload
    pub inception: PersistedIdInception,
    // The key specifies the version of event
    pub events: HashMap<u64, PersistedIdEvent>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct IdEntry {
    pub view: IdView,
    pub identity: PersistedId,
    pub is_provided: bool,
    pub subscribers: Vec<String>,
    pub messages: HashMap<Cid, Vec<u8>>,
}
