use futures::{channel::mpsc::Sender, SinkExt};
use idp2p_common::cbor;
use tracing::info;

use libp2p::{
    gossipsub::{IdentTopic, TopicHash},
    PeerId,
};
use std::{collections::HashMap, str::FromStr, sync::Arc};

use crate::{
    error::HandleError,
    message::{
        IdGossipMessageKind, IdMessageHandlerRequestKind, IdMessageHandlerResponseKind,
        IdMessageNotifyKind,
    },
    model::{IdEntry, IdEntryKind, IdMessage, IdStore, IdVerifier},
    PersistedIdEvent, PersistedIdInception,
};

pub struct IdMessageHandler<S: IdStore, V: IdVerifier> {
    store: Arc<S>,
    verifier: Arc<V>,
    sender: Sender<IdMessageHandlerCommand>,
}

pub enum IdMessageHandlerCommand {
    Publish {
        topic: TopicHash,
        payload: IdGossipMessageKind,
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
        providers: Vec<String>
    ) -> Result<Option<Vec<u8>>, HandleError> {
        use IdGossipMessageKind::*;
        let mut id_entry: IdEntry = self
            .store
            .get_id(topic.as_str())
            .await?
            .ok_or(HandleError::IdNotFound(topic.to_string()))?;
        let payload = cbor::decode(payload)?;
        info!("Received gossip message: {topic}");

        match payload {
            Resolve => {
                if id_entry.kind != IdEntryKind::Following {
                    let cmd = IdMessageHandlerCommand::Publish {
                        topic: topic.to_owned(),
                        payload: IdGossipMessageKind::NotifyMessage {
                            id: id_entry.inception.id.to_string(),
                            providers: providers,
                            kind: IdMessageNotifyKind::ProvideId,
                        },
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
                    IdMessageNotifyKind::ProvideId => {}
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
                    }
                    _ => {}
                };

                return Ok(None);
            }
            Other(payload) => {
                return Ok(Some(payload));
            }
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
                self.create_id(IdEntryKind::Following, inception, events)
                    .await?;
            }
        }
        Ok(())
    }

    pub async fn create_id(
        &self,
        kind: IdEntryKind,
        inception: PersistedIdInception,
        events: HashMap<String, PersistedIdEvent>,
    ) -> Result<(), HandleError> {
        let mut id_projection = self.verifier.verify_inception(&inception).await?;
        for (_, event) in events.clone() {
            id_projection = self.verifier.verify_event(&id_projection, &event).await?;
        }
        let id = inception.id.clone();
        let entry = IdEntry {
            kind: kind,
            projection: id_projection,
            inception: inception,
            events: events,
        };
        let p2p_id = IdentTopic::new(id).hash().to_string();
        self.store.set_id(&p2p_id, &entry).await?;
        Ok(())
    }
}
