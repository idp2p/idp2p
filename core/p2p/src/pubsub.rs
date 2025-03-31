use crate::{
    error::HandlePubsubMessageError,
    id_key,
    idp2p::p2p::{
        p2p_host,
        types::{P2pEvent, P2pPublishEvent, P2pPutEvent},
    },
    model::{IdEntry, IdEntryKind, IdMessageDirection, PendingResolve},
};
use alloc::{string::String, vec::Vec};
use idp2p_common::cbor;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum IdPubsubMessageKind {
    // Resolve identity - takes an challenge
    Resolve(Vec<u8>),
    // Provide identity, takes a list of providers
    Provide {
        challenge: Vec<u8>,
        providers: Vec<String>,
    },
    // Notify an identity event
    NotifyEvent(Vec<u8>),
    // Notify message(this means you have a message)
    NotifyMessage {
        id: String,
        providers: Vec<String>,
        direction: IdMessageDirection,
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
                if id_entry.kind != IdEntryKind::Following {
                    let pending = P2pPutEvent {
                        key: id_key!(topic).to_string(),
                        value: cbor::encode(&PendingResolve {
                            challenge: challenge,
                            providers: vec![],
                            id: topic.to_string(),
                            timestamp: 0,
                            verifier: vec![],
                        }),
                    };
                    events.push(P2pEvent::Put(pending));
                    let provide = P2pPublishEvent {
                        topic: id_key!(topic).to_string(),
                        payload: cbor::encode(&PendingResolve {
                            challenge: vec![],
                            providers: vec![],
                            id: "".to_string(),
                            timestamp: 0,
                            verifier: vec![],
                        }),
                    };
                    events.push(P2pEvent::Publish(provide));
                }
            }
        }
        IdPubsubMessageKind::Provide {
            challenge: _challenge,
            providers: _providers,
        } => {}
        IdPubsubMessageKind::NotifyEvent(data) => {
            if let Some(id_entry) = p2p_host::get(id_key!(topic))? {
                let id_entry: IdEntry = cbor::decode(&id_entry)?;
                //p2p_host::verify_event(component, state, event)
            }
        }
        IdPubsubMessageKind::NotifyMessage {
            id: _id,
            providers: _providers,
            direction: _dir,
        } => {}
    }
    Ok(events)
}
