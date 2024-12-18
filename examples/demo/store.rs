use std::{collections::HashMap, sync::{Arc, Mutex}};

use anyhow::Result;
use cid::Cid;
use idp2p_common::cbor;
use idp2p_p2p::model::{IdEntry, IdMessage, IdStore};
use serde::{de::DeserializeOwned, Serialize};
use wasmtime::component::Component;

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

    pub async fn get_msg(&self, id: &str) -> Result<Option<IdMessage>> {
        let msg_key = format!("/messages/{}", id);
        let msg = self.get(&msg_key).await?;
        Ok(msg)
    }

    pub async fn set_msg(&self, id: &str, entry: &IdMessage) -> Result<()> {
        let msg_key = format!("/messages/{}", id);
        self.set(&msg_key, entry).await?;
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

pub struct InMemoryIdStore(pub Arc<InMemoryKvStore>);

impl IdStore for InMemoryIdStore {
    async fn get_id(&self, id: &Cid) -> Result<Option<IdEntry>> {
        self.0.get_id(&id.to_string()).await
    }

    async fn get_msg(&self, id: &Cid) -> Result<Option<IdMessage>> {
        self.0.get_msg(&id.to_string()).await
    }

    async fn set_id(&self, id: &Cid, value: &IdEntry) -> Result<()> {
        self.0.set_id(&id.to_string(), value).await
    }

    async fn set_msg(&self, id: &Cid, value: &IdMessage) -> Result<()> {
        self.0.set_msg(&id.to_string(), value).await
    }

    async fn get_verifiers() -> Result<Vec<Component>> {
        todo!()
    }
}
