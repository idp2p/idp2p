use anyhow::Result;
use cid::Cid;
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
    model::{IdEntry, IdKind, IdMessage, IdTopic},
    store::KvStore,
    IdView, Idp2pId, PersistedIdEvent, PersistedIdInception,
};

pub struct IdMessageHandler<S: KvStore> {
    kv: Arc<S>,
    engine: Engine,
    id_components: Arc<Mutex<HashMap<u64, Component>>>,
}

pub enum IdHandlerGossipCommand {
    Publish {
        topic: TopicHash,
        payload: Vec<u8>,
    },
    Request {
        peer: PeerId,
        message_id: String,
    },
    None
}

impl<S: KvStore> IdMessageHandler<S> {
    pub fn new(
        kv: Arc<S>
    ) -> Result<Self> {
        let engine = Engine::new(Config::new().wasm_component_model(true))?;

        let components = HashMap::new();
        // Load components from store
        let id_components = Arc::new(Mutex::new(components));
        let handler = Self {
            kv,
            engine,
            id_components
        };
        Ok(handler)
    }

    pub async fn handle_gossip_message(
        &self,
        topic: &TopicHash,
        msg: &IdGossipMessageKind,
    ) -> Result<IdHandlerGossipCommand> {
        use IdGossipMessageKind::*;
        let topic_key = format!("/topics/{}", topic);
        let id_topic: Vec<u8> = self
            .kv
            .get(&topic_key)
            .map_err(anyhow::Error::msg)?
            .ok_or(anyhow::anyhow!("No topic found"))?;
        let id_topic: IdTopic = cbor::decode(&id_topic)?;
        match id_topic {
            IdTopic::Client => {
                let (mut id_entry, id_key) = self.get_id(topic.as_str())?;
                match msg {
                    Resolve => {
                        return Ok(IdHandlerGossipCommand::Publish {
                                topic: topic.to_owned(),
                                payload: id_entry.identity.id,
                            });
                    }
                    NotifyEvent { version, event } => {
                        let view = self.verify_event(*version, &id_entry.view, &event)?;
                        id_entry.view = view;
                        self.kv.put(&id_key, &cbor::encode(&id_entry)?)?;
                    }
                    NotifyMessage { id, providers } => {
                        return Ok(IdHandlerGossipCommand::Request {
                                peer: PeerId::from_str(&providers.get(0).unwrap())?,
                                message_id: id.to_string(),
                            });
                    }
                    _ => {}
                }
            }
            IdTopic::Subscription => {
                let (mut id_entry, id_key) = self.get_id(topic.as_str())?;
                match msg {
                    NotifyEvent { version, event } => {
                        let view = self.verify_event(*version, &id_entry.view, &event)?;
                        id_entry.view = view;
                        self.kv.put(&id_key, &cbor::encode(&id_entry)?)?;
                    }
                    Provide { id } => {
                        let mut view = self.verify_inception(id.version, &id.inception)?;
                        for (version, event) in id.events.clone() {
                            view = self.verify_event(version, &view, &event)?;
                        }
                        let entry = IdEntry {
                            view,
                            identity: id.clone(),
                            kind: IdKind::Subscriber,
                        };
                        self.kv.put(&id_key, &cbor::encode(&entry)?)?;
                    }
                    _ => {}
                }
            }
            IdTopic::Custom => {}
        }

        Ok(IdHandlerGossipCommand::None)
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

            if id.view.mediators.contains(&peer.to_bytes()) {
                return Ok(message.payload.clone());
            }
        }

        anyhow::bail!("Unauthorized message");
    }

    pub async fn handle_response_message(&self, from: Cid, message_id: Cid, payload: Vec<u8>) -> Result<()> {
        let msg = IdMessage{
            from,
            to: vec![],
            payload
        };
        let msg = cbor::encode(&msg)?;
        self.kv.put(&format!("/messages/{}", message_id), &msg)?;
        Ok(())
    }

    fn get_id(&self, id: &str) -> Result<(IdEntry, String)> {
        let id_key = format!("/identities/{}", id);
        let id_entry: Vec<u8> = self
            .kv
            .get(&id_key)
            .map_err(anyhow::Error::msg)?
            .ok_or(anyhow::anyhow!("No topic found"))?;
        let id_entry: IdEntry = cbor::decode(&id_entry)?;
        Ok((id_entry, id_key))
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
