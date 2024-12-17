use std::{collections::HashMap, sync::Mutex};

use anyhow::Result;
use idp2p_common::cbor;
use serde::{de::DeserializeOwned, Serialize};

#[trait_variant::make(Send)]
pub trait KvStore {
    async fn get<T: DeserializeOwned>(&self, key: &str) -> Result<Option<T>>;
    async fn set<T: Serialize + Send + Sync>(&self, key: &str, value: &T) -> Result<()>;
    async fn exists(&self, key: &str) -> Result<bool>;
}

pub struct InMemoryKvStore {
    state: Mutex<HashMap<String, Vec<u8>>>,
}

impl InMemoryKvStore {
    pub fn new() -> Self {
        Self {
            state: Mutex::new(HashMap::new()),
        }
    }
}

impl KvStore for InMemoryKvStore {
    async fn exists(&self, key: &str) -> Result<bool> {
        let state = self.state.lock().unwrap();
        Ok(state.contains_key(key))
    }

    async fn set<T: Serialize + Send + Sync>(&self, key: &str, value: &T) -> Result<()> {
        let mut state = self.state.lock().unwrap();
        state.insert(key.to_owned(), cbor::encode(value)?);
        Ok(())
    }

    async fn get<T: DeserializeOwned>(&self, key: &str) -> Result<Option<T>> {
        let state = self.state.lock().unwrap();
        if let Some(value) = state.get(key) {
            return Ok(Some(cbor::decode(&value)?));
        }
        Ok(None)
    }
}