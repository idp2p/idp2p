use anyhow::Result;
use cid::Cid;
use futures::{channel::mpsc, SinkExt, StreamExt};
use idp2p_common::cbor;

use libp2p::{gossipsub::TopicHash, PeerId};
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};
use wasmtime::{
    component::{Component, Linker},
    Config, Engine, Store,
};

use crate::{
    entry::{IdEntry, PersistedId},
    store::KvStore,
    IdView, Idp2pId, PersistedIdEvent, PersistedIdInception,
};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum IdMessageRequest {
    Get(String),
    Provide(Vec<u8>),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum IdGossipMessageKind {
    // Resolve identity
    Resolve,
    // Provide an identity document
    Provide {
        id: PersistedId,
    },
    // Notify an identity event
    NotifyEvent {
        version: u64,
        event: PersistedIdEvent,
    },
    // Notify message
    NotifyMessage {
        id: Cid,
        providers: Vec<String>,
    },
}

pub enum IdHandlerMessage {
    Gossipsub { topic: TopicHash, payload: Vec<u8> },
    Request(Vec<u8>),
    Response(Vec<u8>),
}

pub enum IdHandlerEvent {
    Publish {
        topic: TopicHash,
        payload: Vec<u8>,
    },
    Request {
        peer: PeerId,
        message_id: String,
    },
    Respond {
        message_id: String,
        payload: Vec<u8>,
    },
    Set {
        key: String,
        value: Vec<u8>,
    },
}

pub struct IdMessageHandler<S: KvStore> {
    kv: Arc<S>,
    engine: Engine,
    id_components: Arc<Mutex<HashMap<u64, Component>>>,
    event_sender: mpsc::Sender<IdHandlerEvent>,
    msg_receiver: mpsc::Receiver<IdHandlerMessage>,
}

impl<S: KvStore> IdMessageHandler<S> {
    pub fn new(
        kv: Arc<S>,
        event_sender: mpsc::Sender<IdHandlerEvent>,
        msg_receiver: mpsc::Receiver<IdHandlerMessage>,
    ) -> Result<Self> {
        let engine = Engine::new(Config::new().wasm_component_model(true))?;

        let components = HashMap::new();
        // Load components from store
        let id_components = Arc::new(Mutex::new(components));
        let handler = Self {
            kv,
            engine,
            id_components,
            event_sender,
            msg_receiver,
        };
        Ok(handler)
    }

    pub async fn handle_gossip_message(&mut self, topic: &TopicHash, msg: &[u8]) -> Result<()> {
        use IdGossipMessageKind::*;
        let message: IdGossipMessageKind = cbor::decode(msg)?;
        let id_key = format!("/identities/{}", topic);
        if let Some(id_entry) = self.kv.get(&id_key).map_err(anyhow::Error::msg)? {
            let mut id_entry: IdEntry = cbor::decode(&id_entry)?;

            match message {
                Resolve => {
                    if id_entry.is_provided {
                        let _ = self.event_sender.send(IdHandlerEvent::Publish {
                            topic: topic.to_owned(),
                            payload: id_entry.identity.id,
                        });
                    }
                }
                NotifyEvent { version, event } => {
                    let view = self.verify_event(version, &id_entry.view, &event)?;
                    id_entry.view = view;
                    self.kv.put("key", &cbor::encode(&id_entry)?)?;
                }
                NotifyMessage {
                    id: _,
                    providers: _,
                } => {
                    //
                }
                _ => {}
            }
        } else {
            match message {
                Provide { id } => {
                    let mut view = self.verify_inception(id.version, &id.inception)?;
                    for (version, event) in id.events.clone() {
                        view = self.verify_event(version, &view, &event)?;
                    }
                    let entry = IdEntry {
                        identity: id,
                        view: view,
                        is_provided: false,
                        subscribers: vec![],
                        messages: HashMap::new(),
                    };
                    self.kv.put("key", &cbor::encode(&entry)?)?;
                }
                _ => {}
            }
        }

        Ok(())
    }

    pub async fn handle_request(
        &mut self,
        peer: PeerId,
        message_id: String,
        msg: &[u8],
    ) -> Result<()> {
        Ok(())
    }

    pub async fn run(mut self) {
        loop {
            tokio::select! {
                msg = self.msg_receiver.next() => match msg {
                    Some(msg) => {
                        match msg {
                            IdHandlerMessage::Gossipsub { topic, payload } => todo!(),
                            IdHandlerMessage::Request(vec) => todo!(),
                            IdHandlerMessage::Response(vec) => todo!(),
                        }
                    },
                    None =>  return,
                },
            }
        }
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

/*

        /*id.verify_event(&message)?;
        let memory = instance
            .get_memory(&mut store, "memory")
            .ok_or_else(|| anyhow::anyhow!(""))?;
        let alloc_func = instance.get_typed_func::<i32, i32>(&mut store, "alloc")?;
        let de_alloc_func = instance.get_typed_func::<i32, ()>(&mut store, "de_alloc")?;
        let input_bytes = message.payload.clone();
        let input_bytes_len = message.payload.len() as i32;
        let input_bytes_ptr = alloc_func.call(&mut store, input_bytes_len)?;
        memory
            .write(&mut store, input_bytes_ptr as usize, &input_bytes)
            .unwrap();
        let func = instance.get_typed_func::<(i32, i32), (i32, i32)>(&mut store, "handle")?;
        let result = func.call(&mut store, (input_bytes_ptr, input_bytes_len))?;
        de_alloc_func.call(&mut store, result.0)?;*/
pub fn handle_gossip_message(topic: &str, msg: &[u8]) -> anyhow::Result<Vec<IdPublishEvent>> {
    let msg: IdGossipMessageKind = decode(&msg)?;
    let id_key = format!("/identities/{}", topic);
    let mut commands = Vec::new();
    if let Some(id_entry) = get(&id_key).map_err(anyhow::Error::msg)? {
        let id_entry: IdEntry = decode(&id_entry)?;
        match msg {
            IdGossipMessageKind::Resolve => {
                if id_entry.provided {
                    commands.push(IdPublishEvent {
                        topic: topic.to_string(),
                        payload: vec![],
                    });
                }
            }
            IdGossipMessageKind::NotifyEvent { event } => {}
            IdGossipMessageKind::NotifyMessage { id, providers } => {
                //
            }
            _ => {}
        }
    } else {
        match msg {
            IdGossipMessageKind::Provide { id } => {}
            _ => {}
        }
    }
    Ok(commands)
} */
