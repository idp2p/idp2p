use std::{collections::HashMap, sync::Mutex};

use serde::{de::DeserializeOwned, Serialize};
use anyhow::Result;
use crate::cbor::{decode, encode};

pub trait Store {
    fn get<T: DeserializeOwned>(&self, key: &str) -> Result<Option<T>>;
    fn put<T: Serialize>(&self, key: &str, value: T) -> Result<()>;
}

pub struct InMemoryStore {
    pub state: Mutex<HashMap<String, Vec<u8>>>,
}

impl InMemoryStore {
    pub fn new() -> Self {
        Self {
            state: Mutex::new(HashMap::new()),
        }
    }
}

impl Store for InMemoryStore {
    fn get<T: DeserializeOwned>(&self, key: &str) -> Result<Option<T>> {
        let state = self.state.lock().unwrap();
        if let Some(bytes) = state.get(key) {
            return Ok(Some(decode(bytes)?));
        }
        Ok(None)
    }

    fn put<T: Serialize>(&self, key: &str, value: T) -> Result<()> {
        let mut state = self.state.lock().unwrap();
        let bytes = encode(&value)?;
        state.insert(key.to_owned(), bytes);
        Ok(())
    }
}