use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

use idp2p_common::cbor;
use libp2p::PeerId;
use serde::{Deserialize, Serialize, de::DeserializeOwned};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct User {
    pub id: String,
    pub peer: PeerId,
    pub username: String,
}

pub struct KvState {
    pub current: User,
    pub pair: Option<User>,
    pub kvdb: HashMap<String, Vec<u8>>,
}

pub struct InMemoryKvStore {
    state: Mutex<KvState>,
}

impl InMemoryKvStore {
    pub fn new(username: &str, id: &str, peer: PeerId) -> Self {
        let state = KvState {
            current: User {
                username: username.to_string(),
                id: id.to_string(),
                peer: peer,
            },
            pair: None,
            kvdb: HashMap::new(),
        };
        Self {
            state: Mutex::new(state),
        }
    }

    pub async fn set<T: Serialize + Send + Sync>(
        &self,
        key: &str,
        value: &T,
    ) -> anyhow::Result<()> {
        let mut state = self.state.lock().unwrap();
        state.kvdb.insert(key.to_owned(), cbor::encode(value));
        Ok(())
    }

    pub async fn get<T: DeserializeOwned>(&self, key: &str) -> anyhow::Result<Option<T>> {
        let state = self.state.lock().unwrap();
        if let Some(value) = state.kvdb.get(key) {
            return Ok(Some(cbor::decode(&value)?));
        }
        Ok(None)
    }

}

pub struct InMemoryIdStore(pub Arc<InMemoryKvStore>);

/*impl IdStore for InMemoryIdStore {
    async fn get_id(&self, id: &str) -> Result<Option<IdEntry2>, HandleError> {
        let id_key = format!("/identities/{}", id);
        let id = self.0.get(&id_key).await?;
        Ok(id)
    }

    async fn get_msg(&self, id: &str) -> Result<Option<IdMessage>, HandleError> {
        let msg_key = format!("/messages/{}", id);
        let msg = self.0.get(&msg_key).await?;
        Ok(msg)
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
}*/
