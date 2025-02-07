use futures::{channel::mpsc::Sender, SinkExt};
use idp2p_common::{cbor, id::Id};

use libp2p::{gossipsub::TopicHash, PeerId};
use std::{str::FromStr, sync::Arc};

use crate::{
    error::HandleError,
    message::{IdGossipMessageKind, IdMessageHandlerRequestKind, IdMessageHandlerResponseKind, IdMessageNotifyKind},
    model::{IdEntry, IdEntryKind, IdMessage, IdStore, IdVerifier},
};

pub struct IdMessageHandler<S: IdStore, V: IdVerifier> {
    store: Arc<S>,
    verifier: Arc<V>,
    sender: Sender<IdMessageHandlerCommand>,
}

pub enum IdMessageHandlerCommand {
    Publish {
        topic: TopicHash,
        payload: Vec<u8>,
    },
    Request {
        peer: PeerId,
        payload: IdMessageHandlerRequestKind,
    },
}

impl<S: IdStore, V: IdVerifier> IdMessageHandler<S, V> {
    pub fn new(store: Arc<S>, verifier: Arc<V>, sender: Sender<IdMessageHandlerCommand>) -> Self {
        Self {
            store,
            verifier,
            sender,
        }
    }

    pub async fn handle_gossip_message(
        &mut self,
        topic: &TopicHash,
        payload: &[u8],
    ) -> Result<Option<Vec<u8>>, HandleError> {
        use IdGossipMessageKind::*;
        let id = Id::from_str(topic.as_str())?;
        if id.kind == "id" {
            let mut id_entry: IdEntry = self
                .store
                .get_id(topic.as_str())
                .await?
                .ok_or(HandleError::IdNotFound(topic.to_string()))?;
            let payload = cbor::decode(payload)?;
            match payload {
                Resolve => {
                    if id_entry.kind != IdEntryKind::Following {
                        let cmd = IdMessageHandlerCommand::Publish {
                            topic: topic.to_owned(),
                            payload: cbor::encode(&id_entry),
                        };
                        self.sender.send(cmd).await.expect("Error sending message");
                    }

                    return Ok(None);
                }
                NotifyEvent(event) => {
                    if id_entry.projection.event_id != event.id {
                        let projection = self
                            .verifier
                            .verify_event(&id_entry.projection, &event)
                            .await?;
                        id_entry.projection = projection;
                        self.store.set_id(topic.as_str(), &id_entry).await?;
                    }

                    return Ok(None);
                }
                NotifyMessage {
                    id,
                    providers,
                    kind,
                } => {
                    match kind {
                        IdMessageNotifyKind::ProvideId => {

                        },
                        IdMessageNotifyKind::SendMessage => {
                            if id_entry.kind != IdEntryKind::Following {
                                let payload = IdMessageHandlerRequestKind::MessageRequest {
                                    id: id_entry.inception.id.to_string(),
                                    message_id: id.to_string(),
                                };
                                let cmd = IdMessageHandlerCommand::Request {
                                    peer: PeerId::from_str(&providers.get(0).unwrap()).unwrap(),
                                    payload,
                                };
                                self.sender.send(cmd).await.expect(" Error sending message");
                            }
                        },
                        _=> {}
                    };

                    return Ok(None);
                }
                Other(payload) => {
                    return Ok(Some(payload));
                }
            }
        } else {
            return Err(HandleError::IdNotFound(topic.to_string()));
        }
    }

    pub async fn handle_request_message(
        &self,
        peer_id: PeerId,
        req: IdMessageHandlerRequestKind,
    ) -> Result<IdMessageHandlerResponseKind, HandleError> {
        match req {
            IdMessageHandlerRequestKind::MessageRequest { id, message_id } => {
                let peer = self
                    .store
                    .get_peer(&peer_id.to_string())
                    .await?
                    .ok_or(HandleError::PeerNotFound(peer_id.to_string()))?;
                let message = self
                    .store
                    .get_msg(&message_id)
                    .await?
                    .ok_or(HandleError::IdNotFound(message_id))?;
                let id_entry = self
                    .store
                    .get_id(&id)
                    .await?
                    .ok_or(HandleError::IdNotFound(id.to_string()))?;
                if message.to.contains(&id) && peer.owner == id_entry.projection.id {
                    return Ok(IdMessageHandlerResponseKind::MessageResponse(
                        message.payload,
                    ));
                }
                return Err(HandleError::PeerNotFound(peer_id.to_string()));
            }
            IdMessageHandlerRequestKind::IdRequest(id) => {
                let id_entry = self
                    .store
                    .get_id(&id)
                    .await?
                    .ok_or(HandleError::IdNotFound(id.clone()))?;
                return Ok(IdMessageHandlerResponseKind::IdResponse {
                    inception: id_entry.inception,
                    events: id_entry.events,
                });
            }
        }
    }

    pub async fn handle_response_message(
        &self,
        from: &str,
        message_id: &str,
        message_body: IdMessageHandlerResponseKind,
    ) -> Result<(), HandleError> {
        match message_body {
            IdMessageHandlerResponseKind::MessageResponse(payload) => {
                let msg = IdMessage {
                    from: from.to_string(),
                    to: vec![],
                    payload,
                };
                self.store.set_msg(message_id, &msg).await?;
            }
            IdMessageHandlerResponseKind::IdResponse { inception, events } => {
                let mut id_projection = self.verifier.verify_inception(&inception).await?;
                for (_, event) in events.clone() {
                    id_projection = self.verifier.verify_event(&id_projection, &event).await?;
                }
                let id = inception.id.clone();
                let entry = IdEntry {
                    kind: IdEntryKind::Following,
                    projection: id_projection,
                    inception: inception,
                    events: events,
                };
                self.store.set_id(&id, &entry).await?;
            }
        }
        Ok(())
    }
}
