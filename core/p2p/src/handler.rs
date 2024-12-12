use anyhow::Result;
use cid::Cid;
use futures::{channel::mpsc, SinkExt, StreamExt};
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
    message::{self, IdGossipMessageKind},
    model::{IdEntry, IdKind, IdMessage, IdTopic},
    store::KvStore,
    IdView, Idp2pId, PersistedIdEvent, PersistedIdInception,
};

pub enum IdHandlerInboundEvent {
    Gossipsub { topic: TopicHash, payload: Vec<u8> },
    Request { peer: PeerId, message_id: Cid },
    Response(Vec<u8>),
}

pub enum IdHandlerOutboundEvent {
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
    event_sender: mpsc::Sender<IdHandlerOutboundEvent>,
    event_receiver: mpsc::Receiver<IdHandlerInboundEvent>,
}

impl<S: KvStore> IdMessageHandler<S> {
    pub fn new(
        kv: Arc<S>,
        event_sender: mpsc::Sender<IdHandlerOutboundEvent>,
        event_receiver: mpsc::Receiver<IdHandlerInboundEvent>,
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
            event_receiver,
        };
        Ok(handler)
    }

    async fn handle_gossip_message(&mut self, topic: &TopicHash, msg: &[u8]) -> Result<()> {
        use IdGossipMessageKind::*;
        let topic_key = format!("/topics/{}", topic);
        let id_topic: Vec<u8> = self
            .kv
            .get(&topic_key)
            .map_err(anyhow::Error::msg)?
            .ok_or(anyhow::anyhow!("No topic found"))?;
        let id_topic: IdTopic = cbor::decode(&id_topic)?;
        let message: IdGossipMessageKind = cbor::decode(msg)?;
        match id_topic {
            IdTopic::Client => {
                let (mut id_entry, id_key) = self.get_id(topic.as_str())?;
                match message {
                    Resolve => {
                        self.event_sender
                            .send(IdHandlerOutboundEvent::Publish {
                                topic: topic.to_owned(),
                                payload: id_entry.identity.id,
                            })
                            .await?;
                    }
                    NotifyEvent { version, event } => {
                        let view = self.verify_event(version, &id_entry.view, &event)?;
                        id_entry.view = view;
                        self.kv.put(&id_key, &cbor::encode(&id_entry)?)?;
                    }
                    NotifyMessage { id, providers } => {
                        self.event_sender
                            .send(IdHandlerOutboundEvent::Request {
                                peer: PeerId::from_str(&providers.get(0).unwrap())?,
                                message_id: id.to_string(),
                            })
                            .await?;
                    }
                    _ => {}
                }
            }
            IdTopic::Subscription => {
                let (mut id_entry, id_key) = self.get_id(topic.as_str())?;
                match message {
                    NotifyEvent { version, event } => {
                        let view = self.verify_event(version, &id_entry.view, &event)?;
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
                            identity: id,
                            kind: IdKind::Subscriber,
                        };
                        self.kv.put(&id_key, &cbor::encode(&entry)?)?;
                    }
                    _ => {}
                }
            }
            IdTopic::Custom => {}
        }

        Ok(())
    }

    async fn handle_request(&mut self, peer: PeerId, message_id: Cid) -> Result<()> {
        let message_id = format!("/messages/{}", message_id);
        let message: Vec<u8> = self
            .kv
            .get(&message_id)
            .map_err(anyhow::Error::msg)?
            .ok_or(anyhow::anyhow!("No message found"))?;
        let message: IdMessage = cbor::decode(&message)?;
        let (id, _) = self.get_id(&message.to.to_string())?;
        if id.view.mediators.contains(&peer.to_bytes()) {
            self.event_sender
                .send(IdHandlerOutboundEvent::Respond {
                    message_id,
                    payload: message.payload,
                })
                .await?;
        }
        Ok(())
    }

    pub async fn run(mut self) {
        use IdHandlerInboundEvent::*;
        loop {
            tokio::select! {
                msg = self.event_receiver.next() => match msg {
                    Some(msg) => {
                        match msg {
                            Gossipsub { topic, payload } => {
                                self.handle_gossip_message(&topic, &payload).await.expect("Failed to handle gossip message");
                            },
                            Request { peer, message_id } => {
                                self.handle_request(peer, message_id).await.expect("Failed to handle request");
                            },
                            Response(msg) => todo!(),
                        }
                    },
                    None =>  return,
                },
            }
        }
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

/*let id_key = format!("/identities/{}", topic);
if let Some(id_entry) = self.kv.get(&id_key).map_err(anyhow::Error::msg)? {
    let id_entry: IdEntry = cbor::decode(&id_entry)?;

    match message {
        Resolve => {
            //if id_entry.provided {
            let _ = self.event_sender.send(IdHandlerEvent::Publish {
                topic: topic.to_owned(),
                payload: id_entry.identity.id,
            });
            //}
        }
        NotifyEvent { version, event } => {
            let view = self.verify_event(version, &id_entry.view, &event)?;
            println!("{:?}", view);
        }
        NotifyMessage(msg) => {
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
                view,
                identity: id,
                subscribers: vec![],
            };
            self.kv.put("key", &cbor::encode(&entry)?)?;
        }
        _ => {}
    }
}*/
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
