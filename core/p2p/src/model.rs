use anyhow::Result;
use cid::Cid;
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, sync::Arc};

use crate::{store::KvStore, IdView, PersistedIdEvent, PersistedIdInception};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PersistedId {
    // Identity id
    pub id: Vec<u8>,
    // Specifies the version of inception
    pub version: u64,
    // Inception id and payload
    pub inception: PersistedIdInception,
    // The key specifies the version of event
    pub events: HashMap<u64, PersistedIdEvent>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct IdEntry {
    pub view: IdView,
    pub identity: PersistedId,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct IdMessage {
    pub from: Cid,
    pub to: Vec<Cid>,
    pub payload: Vec<u8>,
}

pub struct IdStore<S: KvStore> {
    kv: Arc<S>,
}

impl<S: KvStore> IdStore<S> {
    pub fn new(kv: Arc<S>) -> Self {
        Self { kv }
    }

    pub async fn get(&self, id: &str) -> Result<Option<IdEntry>> {
        let id_key = format!("/identities/{}", id);
        let id = self.kv.get(&id_key).await?;
        Ok(id)
    }

    pub async fn set(&self, id: &str, entry: &IdEntry) -> Result<()> {
        let id_key = format!("/identities/{}", id);
        self.kv.set(&id_key, entry).await?;
        Ok(())
    } 

}
