use idp2p_common::anyhow::Result;
use idp2p_common::chrono::Utc;
use std::collections::HashMap;
use std::collections::HashSet;
use std::sync::Mutex;
use std::time::Instant;

pub struct IdStoreOptions {
    pub clients: HashMap<String, Vec<String>>,
    pub subscriptions: HashSet<String>,
}

pub struct IdStore {
    pub state: Mutex<IdState>,
}

#[derive(PartialEq, Debug, Clone)]
pub struct IdState {
    pub clients: HashMap<String, Vec<String>>,
    pub subscriptions: HashSet<String>,
}

#[derive(PartialEq, Debug, Clone)]
pub struct Client {
    pub id: String,
}
