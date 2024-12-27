use anyhow::{Ok, Result};
use futures::{channel::mpsc::Sender, SinkExt};
use idp2p_common::cbor;

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
    pub fn new(
        store: Arc<S>,
        verifier: Arc<V>,
        sender: Sender<IdMessageHandlerCommand>,
    ) -> Result<Self> {
        let handler = Self {
            store,
            verifier,
            sender,
        };
        Ok(handler)
    }

    pub async fn handle_gossip_message(
        &mut self,
        topic: &TopicHash,
        payload: &[u8],
    ) -> Result<Option<Vec<u8>>> {
        use IdGossipMessageKind::*;
        match topic.as_str() {
            s if s.starts_with("/idp2p/id/") => {
                let id = s.strip_prefix("/idp2p/id/").unwrap();
                let mut id_entry: IdEntry = self
                    .store
                    .get_id(id)
                    .await?
                    .ok_or(anyhow::anyhow!("Client not found"))?;
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
                    NotifyEvent { version, event } => {
                        let view = self
                            .verifier
                            .verify_event(&version, &id_entry.view, &event)
                            .await?;
                        id_entry.view = view;
                        self.store.set_id(&id, &id_entry).await?;
                        return Ok(None);
                    }
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
                    NotifyMessage { id, providers, direction } => {
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
            }
            s if s.starts_with("/idp2p/id/") => {
                let id = s.strip_prefix("/idp2p/id/").unwrap();
                todo!()
            }
            _ => return todo!(),
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
                    .await
                    .map_err(anyhow::Error::msg)?
                    .ok_or(anyhow::anyhow!("No message found"))?;

                for to in message.to {
                    let id = self.store.get_id(&to).await?.ok_or(anyhow::anyhow!("Invalid id"))?;

                    //if id.view.mediators.contains() {
                        return Ok(IdMessageHandlerResponseKind::MessageResponse(message.payload));
                    //}
                }
            }
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
        Ok(())
    }
}
