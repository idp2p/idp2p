use cid::Cid;
use idp2p_common::{cbor::decode, cid::CidExt};
use serde::{Deserialize, Serialize};

use crate::inception::IdInception;

use super::view::IdView;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PersistedIdInception {
    pub id: Cid,
    pub payload: Vec<u8>,
}

impl PersistedIdInception {
    pub fn from_bytes(bytes: &[u8]) -> anyhow::Result<Self> {
        let inception: PersistedIdInception = decode(bytes)?;
        Ok(inception)    
    }

    pub fn verify(&self) -> anyhow::Result<IdView> {
        let inception = IdInception::from_bytes(&self.payload)?;
        self.id.ensure(self.payload.as_slice())?;
        inception.validate()?;
        let signer_ids: Vec<Cid> = inception
            .next_signers
            .iter()
            .map(|s| s.id.clone())
            .collect();
        let id_snapshot = IdView {
            id: self.id.clone(),
            state: inception.state, 
            event_id: self.id.clone(),
            config: inception.config,
            event_timestamp: inception.timestamp.to_string(),
            next_signers: inception.next_signers,
            used_signers: signer_ids,
            mediators: inception.mediators,
        };

       
        Ok(id_snapshot)
    }

    
}