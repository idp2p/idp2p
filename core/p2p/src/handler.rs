use futures::{channel::mpsc::Sender, SinkExt};
use idp2p_common::{cbor, id::Id};

use libp2p::{gossipsub::TopicHash, PeerId};
use std::{str::FromStr, sync::Arc};

use crate::{
    error::HandleError,
    message::{
        IdGossipMessageKind, IdMessageDirection, IdMessageHandlerRequestKind,
        IdMessageHandlerResponseKind,
    },
    model::{IdEntry, IdEntryKind, IdMessage, IdStore, IdVerifier},
};

pub struct IdMessageHandler<S: IdStore, V: IdVerifier> {
    store: Arc<S>,
    verifier: Arc<V>,
    sender: Sender<IdMessageHandlerCommand>,
}

pub enum IdMessageHandlerCommand {
    Publish { topic: TopicHash, payload: Vec<u8> },
    Request { peer: PeerId, message_id: String },
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
                    if id_entry.kind != IdEntryKind::Subscriber {
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
                    direction,
                } => {
                    match direction {
                        IdMessageDirection::From => {}
                        IdMessageDirection::To => {
                            if id_entry.kind != IdEntryKind::Subscriber {
                                let cmd = IdMessageHandlerCommand::Request {
                                    peer: PeerId::from_str(&providers.get(0).unwrap()).unwrap(),
                                    message_id: id.to_string(),
                                };
                                self.sender.send(cmd).await.expect(" Error sending message");
                            }
                        }
                    };

                    return Ok(None);
                }
                Other(payload) => {
                    return Ok(Some(payload));
                }
            }
        } else {
            todo!()
        }
    }

    pub async fn handle_request_message(
        &self,
        peer: PeerId,
        req: IdMessageHandlerRequestKind,
    ) -> Result<IdMessageHandlerResponseKind, HandleError> {
        match req {
            IdMessageHandlerRequestKind::MessageRequest(message_id) => {
                let message = self
                    .store
                    .get_msg(&message_id)
                    .await?
                    .ok_or(HandleError::IdNotFound(message_id))?;

                for to in message.to {
                    let id = self
                        .store
                        .get_id(&to)
                        .await?
                        .ok_or(HandleError::IdNotFound(to))?;

                    //if id.view.mediators.contains() {
                    return Ok(IdMessageHandlerResponseKind::MessageResponse(
                        message.payload,
                    ));
                    //}
                }
            }
            IdMessageHandlerRequestKind::IdRequest(_) => todo!(),
        }
        todo!()
    }

    pub async fn handle_response_message(
        &self,
        from: &str,
        message_id: &str,
        payload: Vec<u8>,
    ) -> Result<(), HandleError> {
        let msg = IdMessage {
            from: from.to_string(),
            to: vec![],
            payload,
        };
        self.store.set_msg(&message_id, &msg).await?;
        /*
                           Provide { id: pid } => {
                       let mut view = self
                           .verifier
                           .verify_inception(&pid.version, &pid.inception)
                           .await?;
                       for (version, event) in pid.events.clone() {
                           view = self.verifier.verify_event(&version, &view, &event).await?;
                       }
                       let entry = IdEntry {
                           view,
                           identity: pid.clone(),
                           is_client: false,
                       };
                       self.store.set_id(&id, &entry).await?;
                       return Ok(None);
                   }
        */
        Ok(())
    }
}
