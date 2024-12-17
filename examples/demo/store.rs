use std::{collections::HashMap, sync::Mutex};

use anyhow::Result;
use idp2p_common::cbor;
use idp2p_p2p::model::IdEntry;
use serde::{de::DeserializeOwned, Serialize};

use crate::IdUser;

pub struct InMemoryKvStore {
    state: Mutex<HashMap<String, Vec<u8>>>,
}

impl InMemoryKvStore {
    pub fn new() -> Self {
        Self {
            state: Mutex::new(HashMap::new()),
        }
    }

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

    pub async fn get_id(&self, id: &str) -> Result<Option<IdEntry>> {
        let id_key = format!("/identities/{}", id);
        let id = self.get(&id_key).await?;
        Ok(id)
    }

    pub async fn set_id(&self, id: &str, entry: &IdEntry) -> Result<()> {
        let id_key = format!("/identities/{}", id);
        self.set(&id_key, entry).await?;
        Ok(())
    }

    pub async fn get_user(&self, username: &str) -> Result<Option<IdUser>> {
        let user_key = format!("/users/{}", username);
        let user = self.get(&user_key).await?;
        Ok(user)
    }

    pub async fn set_user(&self, username: &str, user: &IdUser) -> Result<()> {
        let user_key = format!("/users/{}", username);
        self.set(&user_key, user).await?;
        Ok(())
    }  
}