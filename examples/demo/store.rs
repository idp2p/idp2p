use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

use idp2p_common::cbor;
use libp2p::PeerId;
use serde::{Deserialize, Serialize, de::DeserializeOwned};

pub struct KvState {
    pub kvdb: HashMap<String, Vec<u8>>,
}

pub struct InMemoryKvStore {
    state: Mutex<KvState>,
}

impl InMemoryKvStore {
    pub fn new() -> Self {
        let state = KvState {
            kvdb: HashMap::new(),
        };
        Self {
            state: Mutex::new(state),
        }
    }

    pub fn set(&self, key: &str, value: &[u8]) {
        let mut state = self.state.lock().unwrap();
        state.kvdb.insert(key.to_owned(), value.to_vec());
    }

    pub fn get(&self, key: &str) -> Option<Vec<u8>> {
        let state = self.state.lock().unwrap();
        if let Some(val) = state.kvdb.get(key){
            return Some(val.to_vec());
        }
        None
    }
}
