use anyhow::{Ok, Result};
use cid::Cid;
use idp2p_common::{cbor::decode, store::KvStore};
use idp2p_id::{event::PersistedIdEvent, PersistedId};
use serde::{Deserialize, Serialize};

use crate::entry::IdEntry;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum IdGossipMessageKind {
    // Resolve identity
    Resolve,
    // Provide an identity document
    Provide { id: PersistedId },
    // Notify an identity event
    NotifyEvent { event: PersistedIdEvent },
    // Notify message
    NotifyMessage { id: Cid, providers: Vec<String> },
}
use libp2p::gossipsub::Behaviour;

impl IdGossipMessageKind {
    pub fn handle<S: KvStore>(&self, gossipsub: &Behaviour, store: &S) -> Result<()> {
        Ok(())
    }
}

/*pub fn handle_message(request:IdRequest) -> anyhow::Result<IdResponse> {
    let msg: IdGossipMessageKind = decode(&request.message)?;
    if let Some(entry) = request.id_entry {
        let entry: IdEntry = decode(&entry)?;
        match &msg {
            IdGossipMessageKind::Resolve => {
                if entry.provided {
                    return Ok(IdResponse { update: None, publish: None });
                }
            },
            IdGossipMessageKind::Provide { doc } => {},
            IdGossipMessageKind::NotifyEvent { event } => {},
            IdGossipMessageKind::NotifyMessage { id, providers } => {},
        }
    }else {
        match &msg {
            IdGossipMessageKind::Provide { doc } => {
                 //id_verifier::verify_inception(version, payload)
            }
            _ => {
                return Ok(IdResponse { update: None, publish: None });
            }
        }
    }
    todo!()
}*/
