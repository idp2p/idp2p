use anyhow::Result;
use futures::{channel::mpsc, StreamExt};
use idp2p_common::message::IdMessage;

use libp2p::{gossipsub::TopicHash, PeerId};
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};
use wasmtime::{
    component::{bindgen, Component, Linker},
    Config, Engine, Store,
};

use crate::store::KvStore;

bindgen!({
    world:"idp2p-id",
    path:  "../id/wit/world.wit",
    additional_derives: [PartialEq, Eq, Hash, Clone, serde::Serialize, serde::Deserialize],
});

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
pub struct PersistedId {
    pub id: Vec<u8>,
    pub inception: PersistedIdInception,
    pub events: Vec<PersistedIdEvent>,
}

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
    Provide { id: PersistedId },
    // Notify an identity event
    NotifyEvent { event: PersistedIdEvent },
    // Notify message
    NotifyMessage { id: Cid, providers: Vec<String> },
}

pub enum IdHandlerMessage {
    GossipMessage(Vec<u8>),
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
    id_compoents: Arc<Mutex<HashMap<String, Component>>>,
    event_sender: mpsc::Sender<IdHandlerEvent>,
    cmd_receiver: mpsc::Receiver<IdHandlerMessage>,
}

impl<S: KvStore> IdMessageHandler<S> {
    pub fn new(
        kv: Arc<S>,
        event_sender: mpsc::Sender<IdHandlerEvent>,
        cmd_receiver: mpsc::Receiver<IdHandlerMessage>,
    ) -> Result<Self> {
        let engine = Engine::new(Config::new().wasm_component_model(true))?;

        let handler = Self {
            kv,
            engine,
            id_compoents: todo!(),
            event_sender,
            cmd_receiver,
        };
        Ok(handler)
    }

    pub async fn handle(&mut self, msg: &[u8]) -> Result<()> {
        let message = IdMessage::from_bytes(msg)?;
        let mut store = Store::new(&self.engine, ());
        let component = self
            .id_compoents
            .lock()
            .unwrap()
            .get(&message.version.to_string())
            .unwrap()
            .clone();
        let (id, _) = Idp2pId::instantiate(&mut store, &component, &Linker::new(&self.engine))?;

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
        Ok(())
    }

    pub async fn run(mut self) {
        loop {
            tokio::select! {
                cmd = self.cmd_receiver.next() => match cmd {
                    Some(cmd) => self.handle(todo!()).await.unwrap(),
                    None =>  return,
                },
            }
        }
    }
}

/*
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
