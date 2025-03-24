use crate::{error::HandlePubsubMessageError, model::IdMessageDirection};
use alloc::{string::String, vec::Vec};
use idp2p_common::cbor;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum IdPubsubMessageKind {
    // Resolve identity
    Resolve,
    // Provide identity, takes a list of providers
    Provide(Vec<String>),
    // Notify an identity event
    NotifyEvent(Vec<u8>),
    // Notify message(this means you have a message)
    NotifyMessage {
        id: String,
        providers: Vec<String>,
        direction: IdMessageDirection,
    },
}

pub fn handle_pubsub_message(msg: &[u8]) -> Result<(), HandlePubsubMessageError> {
    let kind: IdPubsubMessageKind = cbor::decode(msg)?;
    match kind {
        IdPubsubMessageKind::Provide(_providers) => {}
        IdPubsubMessageKind::Resolve => {}
        IdPubsubMessageKind::NotifyEvent(_data) => {}
        IdPubsubMessageKind::NotifyMessage {
            id: _id,
            providers: _providers,
            direction: _dir,
        } => {}
    }
    todo!()
}
