use std::collections::BTreeMap;

use crate::{error::HandlePubsubMessageError, model::IdEntry};
use alloc::{string::String, vec::Vec};
use idp2p_common::cbor;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum IdMessageKind {
    Create {
        id: String,
        inception: Vec<u8>,
        events: BTreeMap<String, Vec<u8>>,
    },
    Join(String),
    Update {
        id: String,
        event: Vec<u8>,
    },
    Resolve(String),
    IdRequest(String),
    IdResponse {
        id: String,
        inception: Vec<u8>,
        events: BTreeMap<String, Vec<u8>>,
    },
    NotifyEvent(Vec<u8>),
}

pub fn handle_message(payload: &[u8]) -> Result<(), HandlePubsubMessageError> {
    let kind: IdMessageKind = cbor::decode(&payload[5..])?;
    match kind {
        IdMessageKind::Create {
            id,
            inception,
            events,
        } => {
            // verify id inception and store it
            // subscribe to {id}
            // subscribe to {id}-self
        },
        IdMessageKind::Join(peer_id) => todo!(),
        IdMessageKind::Update { id, event } => todo!(),
        IdMessageKind::Resolve(_) => todo!(),
        IdMessageKind::IdRequest(_) => todo!(),
        IdMessageKind::IdResponse {
            id,
            inception,
            events,
        } => todo!(),
        IdMessageKind::NotifyEvent(items) => todo!()
    }
    Ok(())
}
/*

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum IdMessageKind {
    Create {
        id: String,
        inception: Vec<u8>,
        events: BTreeMap<String, Vec<u8>>,
    },
    Join(String),
    Update {
        id: String,
        event: Vec<u8>,
    },
    Resolve(String),
    SendMessage {
        id: String,
        payload: Vec<u8>,
    },
    IdRequest(String),
    IdResponse {
        id: String,
        inception: Vec<u8>,
        events: BTreeMap<String, Vec<u8>>,
    },
    NotifyEvent(Vec<u8>),
    NotifyMessage {
        id: String,
        providers: Vec<String>,
    },
    MessageRequest {
        peer_id: String,
        message_id: String,
    },
    MessageResponse(Vec<u8>),
}
pub(crate) fn handle_create_id(
    id: String,
    inception: &[u8],
) -> Result<(), HandlePubsubMessageError> {
    // verify id and store it to a node
    // subscribe to id
    todo!()
}

pub(crate) fn handle_update_id(id: String, event: &[u8]) -> Result<(), HandlePubsubMessageError> {
    // verify event and store it
    // notify event
    todo!()
}

pub(crate) fn handle_resolve_id(id: String) -> Result<(), HandlePubsubMessageError> {
    // subscribe to id
    // request id
    todo!()
}

pub(crate) fn handle_send_message(id: String, msg: &[u8]) -> Result<(), HandlePubsubMessageError> {
    // store the message
    // notify message
    todo!()
}

pub fn handle_message(
    topic: &str,
    payload: &[u8],
) -> Result<Vec<P2pEvent>, HandlePubsubMessageError> {
    let mut events: Vec<P2pEvent> = Vec::new();
    let kind: IdPubsubMessageKind = cbor::decode(&payload[5..])?;
    match kind {
        IdPubsubMessageKind::NotifyEvent(data) => {
            if let Some(id_entry) = p2p_host::get(id_key!(topic))? {
                let mut id_entry: IdEntry = cbor::decode(&id_entry)?;
                let state = cbor::encode(&id_entry.state);
                let state = p2p_host::verify_event("component", &state, &data)?;
                id_entry.state = cbor::decode(&state)?;
            }
        }
        IdPubsubMessageKind::NotifyMessageTo {
            id,
            providers,
        } => {
            let msg_req = P2pRequestEvent {
                peer: providers.first().unwrap().to_string(),
                payload: vec![],
            };
            events.push(P2pEvent::Request(msg_req));
        },
        IdPubsubMessageKind::NotifyMessageFrom {
            id,
            providers,
            proof,
        } => {
            let msg_req = P2pRequestEvent {
                peer: providers.first().unwrap().to_string(),
                payload: vec![],
            };
            events.push(P2pEvent::Request(msg_req));
        },
    }
    Ok(events)
}


match kind {
        IdPubsubMessageKind::Resolve(challenge) => {
            if let Some(id_entry) = p2p_host::get(id_key!(topic))? {
                let id_entry: IdEntry = cbor::decode(&id_entry)?;
                let providers = p2p_host::get(id_providers_key!(topic))?.ok_or(
                    HandlePubsubMessageError::ProviderNotFound(topic.to_string()),
                )?;
                let providers = cbor::decode(&providers)?;

                if id_entry.kind != IdEntryKind::Following {
                    let pending = P2pPutEvent {
                        key: id_key!(topic).to_string(),
                        value: cbor::encode(&PendingResolve {
                            id: topic.to_string(),
                            challenge: challenge.clone(),
                            timestamp: Utc::now().timestamp(),
                        }),
                    };
                    events.push(P2pEvent::Put(pending));
                    let provide = P2pPublishEvent {
                        topic: id_key!(topic).to_string(),
                        payload: cbor::encode(&IdPubsubMessageKind::Provide {
                            challenge,
                            providers,
                        }),
                    };
                    events.push(P2pEvent::Publish(provide));
                }
            }
        }
        IdPubsubMessageKind::Provide {
            challenge,
            providers,
        } => {
            if p2p_host::get(id_key!(topic))?.is_none() {
                // select a provider
                // send a request to provider with verifier
            }
        }
        IdPubsubMessageKind::NotifyEvent(data) => {
            if let Some(id_entry) = p2p_host::get(id_key!(topic))? {
                let mut id_entry: IdEntry = cbor::decode(&id_entry)?;
                let state = cbor::encode(&id_entry.state);
                let state = p2p_host::verify_event("component", &state, &data)?;
                id_entry.state = cbor::decode(&state)?;
            }
        }
        IdPubsubMessageKind::NotifyMessage {
            id: _id,
            providers: _providers,
        } => {}
    }*/
