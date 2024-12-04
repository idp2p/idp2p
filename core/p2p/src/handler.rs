use cid::Cid;
use anyhow::Result;
use futures::{channel::mpsc, StreamExt};
use idp2p_common::{cbor::decode, message::IdMessage};
use idp2p_id::model::{event::PersistedIdEvent, id::PersistedId};
use libp2p::{gossipsub::TopicHash, PeerId};
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};
use wasmtime::{Config, Engine, Instance, Linker, Module, Store};

use crate::store::KvStore;

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

pub enum IdHandlerCommand {
    HandleGossipMessage(Vec<u8>),
    HandleRequest(Vec<u8>),
    HandleResponse(Vec<u8>),
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
    modules: Arc<Mutex<HashMap<String, Module>>>,
    event_sender: mpsc::Sender<IdHandlerEvent>,
    cmd_receiver: mpsc::Receiver<IdHandlerCommand>,
}


impl<S: KvStore> IdMessageHandler<S> {
    pub fn new(
        kv: Arc<S>,
        event_sender: mpsc::Sender<IdHandlerEvent>,
        cmd_receiver: mpsc::Receiver<IdHandlerCommand>,
    ) -> Result<Self> {
        let engine = Engine::new(&Config::new())?;

        let handler = Self {
            kv,
            engine,
            modules: todo!(),
            event_sender,
            cmd_receiver,
            
        };
        Ok(handler)
    }

    pub async fn handle(&mut self, msg: &[u8]) -> Result<()> {
        let message = IdMessage::from_bytes(msg)?;
        let mut store = Store::new(&self.engine, ());
        let module = self
            .modules
            .lock()
            .unwrap()
            .get(&message.version.to_string())
            .unwrap()
            .clone();
        let instance = Instance::new(&mut store, &module, &[])?;
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
        de_alloc_func.call(&mut store, result.0)?;
        Ok(())
    }

    pub(crate) async fn run(mut self) {
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
