use anyhow::Result;
use cid::Cid;
use futures::{
    channel::mpsc::{self, Sender},
    SinkExt,
};
use idp2p_common::cbor;

use libp2p::{gossipsub::TopicHash, PeerId};
use std::{
    collections::HashMap,
    str::FromStr,
    sync::{Arc, Mutex},
};
use wasmtime::{
    component::{Component, Linker},
    Config, Engine, Store,
};

use crate::{
    message::IdGossipMessageKind,
    model::{IdEntry, IdMessage, IdStore},
    store::KvStore,
    topic::IdTopic,
    IdView, Idp2pId, PersistedIdEvent, PersistedIdInception,
};

pub struct IdMessageHandler<S: KvStore> {
    kv: Arc<S>,
    engine: Engine,
    sender: Sender<IdMessageHandlerCommand>,
    id_components: Arc<Mutex<HashMap<u64, Component>>>,
}

pub enum IdMessageHandlerCommand {
    Publish { topic: TopicHash, payload: Vec<u8> },
    Request { peer: PeerId, message_id: String },
}

impl<S: KvStore> IdMessageHandler<S> {
    pub fn new(kv: Arc<S>, sender: Sender<IdMessageHandlerCommand>) -> Result<Self> {
        let engine = Engine::new(Config::new().wasm_component_model(true))?;

        let components = HashMap::new();

        let id_components = Arc::new(Mutex::new(components));
        let handler = Self {
            kv,
            engine,
            sender,
            id_components,
        };
        Ok(handler)
    }

    pub async fn handle_gossip_message(
        &mut self,
        topic: &TopicHash,
        payload: &[u8],
    ) -> Result<Option<Vec<u8>>> {
        use IdGossipMessageKind::*;
        let topic_str = topic.to_string();
        let payload = cbor::decode(payload)?;
        let id_store = IdStore::new(self.kv.clone());
        match payload {
            Resolve => {
                let id_entry = id_store
                    .get(&topic_str)
                    .await?
                    .ok_or(anyhow::anyhow!("Client not found"))?;

                let cmd = IdMessageHandlerCommand::Publish {
                    topic: topic.to_owned(),
                    payload: id_entry.identity.id,
                };

                self.sender.send(cmd).await?;
                return Ok(None);
            }
            NotifyEvent { version, event } => {
                let mut id_entry = id_store
                    .get(&topic_str)
                    .await?
                    .ok_or(anyhow::anyhow!("Subscription not found"))?;
                let view = self.verify_event(version, &id_entry.view, &event)?;
                id_entry.view = view;
                id_store.set(&topic_str, &id_entry).await?;
                return Ok(None);
            }
            Provide { id } => {
                let mut view = self.verify_inception(id.version, &id.inception)?;
                for (version, event) in id.events.clone() {
                    view = self.verify_event(version, &view, &event)?;
                }
                let entry = IdEntry {
                    view,
                    identity: id.clone()
                };
                id_store.set(&topic_str, &entry).await?;
                return Ok(None);
            }
            NotifyMessage { id, providers } => {
                let cmd = IdMessageHandlerCommand::Request {
                    peer: PeerId::from_str(&providers.get(0).unwrap())?,
                    message_id: id.to_string(),
                };
                self.sender.send(cmd).await?;
                return Ok(None);
            },
            Other(payload) => {
                return Ok(Some(payload));
            }
        }
    }

    pub async fn handle_request_message(&self, peer: PeerId, message_id: Cid) -> Result<Vec<u8>> {
        let message_id = format!("/messages/{}", message_id);
        let message: Vec<u8> = self
            .kv
            .get(&message_id)
            .map_err(anyhow::Error::msg)?
            .ok_or(anyhow::anyhow!("No message found"))?;
        let message: IdMessage = cbor::decode(&message)?;
        for to in message.to {
            let (id, _) = self.get_id(&to.to_string())?;

            if id.view.mediators.contains(&peer.to_string()) {
                return Ok(message.payload.clone());
            }
        }

        anyhow::bail!("Unauthorized message");
    }

    pub async fn handle_response_message(
        &self,
        from: Cid,
        message_id: Cid,
        payload: Vec<u8>,
    ) -> Result<()> {
        let msg = IdMessage {
            from,
            to: vec![],
            payload,
        };
        let msg = cbor::encode(&msg)?;
        self.kv.put(&format!("/messages/{}", message_id), &msg)?;
        Ok(())
    }


    fn get_component(&self, version: u64) -> Result<(Idp2pId, Store<()>)> {
        let mut store = Store::new(&self.engine, ());
        let component = self
            .id_components
            .lock()
            .unwrap()
            .get(&version)
            .unwrap()
            .clone();
        let (id, _) = Idp2pId::instantiate(&mut store, &component, &Linker::new(&self.engine))?;
        Ok((id, store))
    }

    fn verify_inception(&self, version: u64, inception: &PersistedIdInception) -> Result<IdView> {
        let (verifier, mut store) = self.get_component(version)?;
        let view = verifier.call_verify_inception(&mut store, inception)??;
        Ok(view)
    }

    fn verify_event(
        &self,
        version: u64,
        view: &IdView,
        event: &PersistedIdEvent,
    ) -> Result<IdView> {
        let (verifier, mut store) = self.get_component(version)?;
        let view = verifier.call_verify_event(&mut store, view, event)??;
        Ok(view)
    }
}
