use anyhow::Result;
use std::{collections::HashMap, sync::Mutex};

pub struct InMemoryKvStore {
    pub state: Mutex<HashMap<String, Vec<u8>>>,
}

impl InMemoryKvStore {
    pub fn new() -> Self {
        Self {
            state: Mutex::new(HashMap::new()),
        }
    }

    pub fn get(&self, key: &str) -> Result<Option<Vec<u8>>> {
        let state = self.state.lock().unwrap();
        if let Some(value) = state.get(key) {
            return Ok(Some(value.to_vec()));
        }
        Ok(None)
    }

    pub fn put(&self, key: &str, value: &[u8]) -> Result<()> {
        let mut state = self.state.lock().unwrap();
        state.insert(key.to_owned(), value.to_vec());
        Ok(())
    }
}