use crate::{error::HandlePubsubMessageError, model::IdMessageDirection};
use alloc::{string::String, vec::Vec};
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
    todo!()
}
