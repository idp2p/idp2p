use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

use idp2p_common::cbor;
use idp2p_p2p::{error::HandleError, model::{IdEntry, IdMessage, IdPeer, IdStore}};
use libp2p::PeerId;
use serde::{de::DeserializeOwned, Serialize};

use crate::user::UserState;

pub struct InMemoryKvStore {
    state: Mutex<HashMap<String, Vec<u8>>>,
}

impl InMemoryKvStore {
    pub fn new() -> Self {
        Self {
            state: Mutex::new(HashMap::new()),
        }
    }

    pub async fn set<T: Serialize + Send + Sync>(&self, key: &str, value: &T) -> anyhow::Result<()> {
        let mut state = self.state.lock().unwrap();
        state.insert(key.to_owned(), cbor::encode(value));
        Ok(())
    }

    pub async fn get<T: DeserializeOwned>(&self, key: &str) -> anyhow::Result<Option<T>> {
        let state = self.state.lock().unwrap();
        if let Some(value) = state.get(key) {
            return Ok(Some(cbor::decode(&value)?));
        }
        Ok(None)
    }

    pub async fn get_current_user(&self) -> anyhow::Result<UserState> {
        let user: UserState = self.get("/current_user").await?.unwrap();
        Ok(user)
    }

    pub async fn set_current_user(&self, user: &UserState) -> anyhow::Result<()> {
        self.set("/current_user", user).await
    }
}

pub struct InMemoryIdStore(pub Arc<InMemoryKvStore>);

impl IdStore for InMemoryIdStore {
    async fn get_id(&self, id: &str) -> Result<Option<IdEntry>, HandleError> {
        let id_key = format!("/identities/{}", id);
        let id = self.0.get(&id_key).await?;
        Ok(id)
    }

    async fn get_msg(&self, id: &str) -> Result<Option<IdMessage>, HandleError> {
        let msg_key = format!("/messages/{}", id);
        let msg = self.0.get(&msg_key).await?;
        Ok(msg)
    }

    async fn get_peer(&self, id: &str) -> Result<Option<IdPeer>, HandleError> {
        let peer_key = format!("/peers/{}", id);
        let peer = self.0.get(&peer_key).await?;
        Ok(peer)
    }

    async fn set_id(&self, id: &str, entry: &IdEntry) -> Result<(), HandleError> {
        let id_key = format!("/identities/{}", id);
        self.0.set(&id_key, entry).await?;
        Ok(())
    }

    async fn set_msg(&self, id: &str, msg: &IdMessage) -> Result<(), HandleError> {
        let msg_key = format!("/messages/{}", id);
        self.0.set(&msg_key, msg).await?;
        Ok(())
    }

    async  fn set_peer(&self,id: &str,value: &IdPeer) -> Result<(),HandleError> {
        let peer_key = format!("/peers/{}", id);
        self.0.set(&peer_key, value).await?;
        Ok(())
    }
}
