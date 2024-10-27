use anyhow::{Ok, Result};
use cid::Cid;
use idp2p_common::cbor::decode;
use serde::{Deserialize, Serialize};

use crate::{entry::IdEntry, exports::idp2p::p2p::id_handler::{IdRequest, IdResponse}, idp2p::p2p::id_verifier};


#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum IdGossipMessageKind {
    // Resolve identity
    Resolve,
    // Provide an identity document
    Provide { doc: Vec<u8> },
    // Notify an identity event
    NotifyEvent { event: Vec<u8> },
    // Notify message
    NotifyMessage { id: Cid, providers: Vec<String> },
}

pub fn handle_message_inner(request:IdRequest) -> anyhow::Result<IdResponse> {    
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
}
