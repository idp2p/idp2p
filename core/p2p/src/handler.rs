use cid::Cid;
use idp2p_id::model::{event::PersistedIdEvent, id::PersistedId};

use crate::{
    entry::IdEntry, exports::idp2p::p2p::id_handler::IdPublishEvent, idp2p::p2p::id_query::*
};
use idp2p_common::cbor::decode;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum IdMessageRequest {
    Get(String),
    Provide(Vec<u8>),
}

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

pub fn handle_gossip_message(topic: &str, msg: &[u8]) -> anyhow::Result<Vec<IdPublishEvent>> {
    let msg: IdGossipMessageKind = decode(&msg)?;
    let id_key = format!("/identities/{}", topic);
    let mut commands = Vec::new();
    if let Some(id_entry) = get(&id_key).map_err(anyhow::Error::msg)? {
        let id_entry: IdEntry = decode(&id_entry)?;
        match msg {
            IdGossipMessageKind::Resolve => {
                if id_entry.provided {
                    commands.push(IdPublishEvent {
                        topic: topic.to_string(),
                        payload: vec![],
                    });
                }
            }
            IdGossipMessageKind::NotifyEvent { event } => {

            }
            IdGossipMessageKind::NotifyMessage { id, providers } => {
                //
            }
            _ => {}
        }
    } else {
        match msg {
            IdGossipMessageKind::Provide { id } => {}
            _ => {}
        }
    }
    Ok(commands)
}