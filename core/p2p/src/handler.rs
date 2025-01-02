use anyhow::Result;

use futures::{channel::mpsc::Sender, SinkExt};
use idp2p_common::{cbor, id::Id};

use libp2p::{gossipsub::TopicHash, PeerId};
use std::{str::FromStr, sync::Arc};

use crate::{
    message::{IdGossipMessageKind, IdMessageHandlerRequestKind, IdMessageHandlerResponseKind},
    model::{IdEntry, IdMessage, IdStore, IdVerifier},
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
    ) -> Result<Option<Vec<u8>>> {
        use IdGossipMessageKind::*;
        let said = Id::from_str(topic.as_str())?;
        if said.kind == "id" {
            let mut id_entry: IdEntry = self
                .store
                .get_id(topic.as_str())
                .await?
                .ok_or(anyhow::anyhow!(""))?;
            let payload = cbor::decode(payload)?;
            match payload {
                Resolve => {
                    let cmd = IdMessageHandlerCommand::Publish {
                        topic: topic.to_owned(),
                        payload: cbor::encode(&id_entry)?,
                    };
                    self.sender.send(cmd).await?;
                    return Ok(None);
                }
                NotifyEvent(event) => {
                    let view = self
                        .verifier
                        .verify_event("", &id_entry.view, &event)
                        .await?;
                    id_entry.view = view;
                    self.store.set_id(&said.version.to_string(), &id_entry).await?;
                    return Ok(None);
                }
                NotifyMessage {
                    id,
                    providers,
                    direction,
                } => {
                    let cmd = IdMessageHandlerCommand::Request {
                        peer: PeerId::from_str(&providers.get(0).unwrap())?,
                        message_id: id.to_string(),
                    };
                    self.sender.send(cmd).await?;
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
    ) -> Result<IdMessageHandlerResponseKind> {
        match req {
            IdMessageHandlerRequestKind::MessageRequest(message_id) => {
                let message = self
                    .store
                    .get_msg(&message_id)
                    .await?
                    .ok_or(anyhow::anyhow!("No message found"))?;

                for to in message.to {
                    let id = self
                        .store
                        .get_id(&to)
                        .await?
                        .ok_or(anyhow::anyhow!("Invalid id"))?;

                    //if id.view.mediators.contains() {
                    return Ok(IdMessageHandlerResponseKind::MessageResponse(
                        message.payload,
                    ));
                    //}
                }
            }
            IdMessageHandlerRequestKind::IdRequest(_) => todo!(),
        }

        anyhow::bail!("Unauthorized message");
    }

    pub async fn handle_response_message(
        &self,
        from: &str,
        message_id: &str,
        payload: Vec<u8>,
    ) -> Result<()> {
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
