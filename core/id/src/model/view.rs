use cid::Cid;
use serde::{Deserialize, Serialize};

use crate::{config::IdConfig, signer::IdSigner};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct IdView {
    pub id: Cid,
    pub state: Cid,
    pub config: IdConfig,
    pub event_id: Cid,
    pub event_timestamp: String,
    pub next_signers: Vec<IdSigner>,
    pub used_signers: Vec<Cid>,
    pub mediators: Vec<String>,
}

impl IdView {
    pub fn to_bytes(&self) -> anyhow::Result<Vec<u8>> {
        todo!()
    }
}
