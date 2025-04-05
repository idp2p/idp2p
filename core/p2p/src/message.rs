use std::collections::BTreeMap;

use crate::{
    error::HandlePubsubMessageError,
    id_key, id_providers_key,
    idp2p::p2p::{
        p2p_host,
        types::{P2pEvent, P2pPublishEvent, P2pPutEvent},
    },
    model::IdEntry,
};
use alloc::{string::String, vec::Vec};
use chrono::Utc;
use idp2p_common::cbor;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum IdPubsubMessageKind {
    // Notify an identity event
    NotifyEvent(Vec<u8>),
    // Notify message(this means you have a message)
    NotifyMessage { id: String, providers: Vec<String> },
    // Broadcast a message
    Broadcast { id: String, providers: Vec<String> },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum IdRequestKind {
    MessageRequest { peer_id: String, message_id: String },
    IdRequest(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum IdResponseKind {
    MessageResponse(Vec<u8>),
    IdResponse {
        id: String,
        inception: Vec<u8>,
        events: BTreeMap<String, Vec<u8>>,
    },
}

pub fn handle_pubsub_message(
    topic: &str,
    payload: &[u8],
) -> Result<Vec<P2pEvent>, HandlePubsubMessageError> {
    let mut events: Vec<P2pEvent> = Vec::new();
    let kind: IdPubsubMessageKind = cbor::decode(&payload[6..])?;
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
    }
    Ok(events)
}
