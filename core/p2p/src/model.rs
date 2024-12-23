use crate::{IdView, PersistedIdEvent, PersistedIdInception};
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PersistedId {
    // Identity id
    pub id: Vec<u8>,
    // Specifies the version of inception
    pub version: String,
    // Inception id and payload
    pub inception: PersistedIdInception,
    // The key specifies the version of event
    pub events: HashMap<String, PersistedIdEvent>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct IdEntry {
    pub view: IdView,
    pub identity: PersistedId,
    pub is_client: bool,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct IdMessage {
    pub from: String,
    pub to: Vec<String>,
    pub payload: Vec<u8>,
}

#[trait_variant::make(Send)]
pub trait IdVerifier {
    async fn verify_inception(
        &self,
        version: &str,
        inception: &PersistedIdInception,
    ) -> Result<IdView>;
    async fn verify_event(
        &self,
        version: &str,
        view: &IdView,
        event: &PersistedIdEvent,
    ) -> Result<IdView>;
}

#[trait_variant::make(Send)]
pub trait IdStore {
    async fn get_id(&self, id: &str) -> Result<Option<IdEntry>>;
    async fn get_msg(&self, id: &str) -> Result<Option<IdMessage>>;
    async fn set_id(&self, id: &str, value: &IdEntry) -> Result<()>;
    async fn set_msg(&self, id: &str, value: &IdMessage) -> Result<()>;
}
