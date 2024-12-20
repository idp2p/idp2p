use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

use anyhow::Result;
use idp2p_common::cbor;
use idp2p_p2p::model::{IdEntry, IdMessage, IdStore};
use serde::{de::DeserializeOwned, Serialize};

use crate::user::IdUser;

pub struct InMemoryKvStore {
    state: Mutex<HashMap<String, Vec<u8>>>,
}

impl InMemoryKvStore {
    pub fn new() -> Self {
        Self {
            state: Mutex::new(HashMap::new()),
        }
    }

    pub async fn set<T: Serialize + Send + Sync>(&self, key: &str, value: &T) -> Result<()> {
        let mut state = self.state.lock().unwrap();
        state.insert(key.to_owned(), cbor::encode(value)?);
        Ok(())
    }

    pub async fn get<T: DeserializeOwned>(&self, key: &str) -> Result<Option<T>> {
        let state = self.state.lock().unwrap();
        if let Some(value) = state.get(key) {
            return Ok(Some(cbor::decode(&value)?));
        }
        Ok(None)
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

    pub async fn get_current_user(&self) -> Result<IdUser> {
        let username: String = self.get("/current-user").await?.unwrap();

        Ok(self.get_user(&username).await?.unwrap())
    }
}

pub struct InMemoryIdStore(pub Arc<InMemoryKvStore>);

impl IdStore for InMemoryIdStore {
    async fn get_id(&self, id: &str) -> Result<Option<IdEntry>> {
        let id_key = format!("/identities/{}", id);
        let id = self.0.get(&id_key).await?;
        Ok(id)
    }

    async fn get_msg(&self, id: &str) -> Result<Option<IdMessage>> {
        let msg_key = format!("/messages/{}", id);
        let msg = self.0.get(&msg_key).await?;
        Ok(msg)
    }

    async fn set_id(&self, id: &str, entry: &IdEntry) -> Result<()> {
        let id_key = format!("/identities/{}", id);
        self.0.set(&id_key, entry).await?;
        Ok(())
    }

    async fn set_msg(&self, id: &str, msg: &IdMessage) -> Result<()> {
        let msg_key = format!("/messages/{}", id);
        self.0.set(&msg_key, msg).await?;
        Ok(())
    }
}
