use crate::{IdView, PersistedIdEvent, PersistedIdInception};
use anyhow::Result;
use cid::Cid;
use serde::{Deserialize, Serialize};
use wasmtime::component::Component;
use std::collections::HashMap;

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
    pub is_client: bool,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct IdMessage {
    pub from: Cid,
    pub to: Vec<Cid>,
    pub payload: Vec<u8>,
}

#[trait_variant::make(Send)]
pub trait IdVerifier {
    async fn verify_inception(
        &self,
        version: u64,
        inception: &PersistedIdInception,
    ) -> Result<IdView>;
    async fn verify_event(
        &self,
        version: u64,
        view: &IdView,
        event: &PersistedIdEvent,
    ) -> Result<IdView>;
}

#[trait_variant::make(Send)]
pub trait IdStore {
    async fn get_id(&self, id: &Cid) -> Result<Option<IdEntry>>;
    async fn get_msg(&self, id: &Cid) -> Result<Option<IdMessage>>;
    async fn get_verifiers() -> Result<Vec<Component>>;
    async fn set_id(&self, id: &Cid, value: &IdEntry) -> Result<()>;
    async fn set_msg(&self, id: &Cid, value: &IdMessage) -> Result<()>;
}
