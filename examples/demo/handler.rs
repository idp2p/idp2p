use anyhow::{Ok, Result};
use exports::idp2p::p2p::id_handler::IdMessageKind;
use futures::{channel::mpsc, StreamExt};
use idp2p::p2p::id_query;
use idp2p_common::message::IdMessage;
use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};
use wasmtime::{
    component::{bindgen, Component, Linker},
    Config, Engine,
};

use crate::{
    command::{IdHandlerCommand, IdNetworkCommand},
    store::KvStore,
};
bindgen!({
    path: "core/p2p/wit",
    additional_derives: [PartialEq, Eq, Hash, Clone, serde::Serialize, serde::Deserialize],
});

struct IdMessageHandlerHost<S: KvStore> {
    kv: Arc<S>,
    engine: Engine,
    id_linker: Linker<()>,
    id_components: Arc<Mutex<HashMap<String, Component>>>,
}

struct IdMessageHandlerState<S: KvStore> {
    host: IdMessageHandlerHost<S>,
}

pub struct IdMessageEventLoop<S: KvStore> {
    kv: Arc<S>,
    engine: Engine,
    id_linker: Linker<()>,
    p2p_linker: Linker<IdMessageHandlerState<S>>,
    id_components: Arc<Mutex<HashMap<String, Component>>>,
    p2p_components: Arc<Mutex<HashMap<String, Component>>>,
    network_cmd_sender: mpsc::Sender<IdNetworkCommand>,
    handler_cmd_receiver: mpsc::Receiver<IdHandlerCommand>,
}

impl<S: KvStore> IdMessageEventLoop<S> {
    pub fn new(
        kv: Arc<S>,
        id_components: Arc<Mutex<HashMap<String, Component>>>,
        p2p_components: Arc<Mutex<HashMap<String, Component>>>,
        network_cmd_sender: mpsc::Sender<IdNetworkCommand>,
        handler_cmd_receiver: mpsc::Receiver<IdHandlerCommand>,
    ) -> Result<Self> {
        let engine = Engine::new(Config::new().wasm_component_model(true))?;
        let id_linker = Linker::new(&engine);
        let mut p2p_linker = Linker::new(&engine);
        id_query::add_to_linker(&mut p2p_linker, |state: &mut IdMessageHandlerState<S>| {
            &mut state.host
        })?;

        let handler = Self {
            kv,
            engine,
            id_linker,
            p2p_linker,
            id_components,
            p2p_components,
            network_cmd_sender,
            handler_cmd_receiver,
        };
        Ok(handler)
    }

    async fn handle_command(&mut self, msg: &[u8]) -> Result<()> {
        let message = IdMessage::from_bytes(msg)?;
        let component = self
            .id_components
            .lock()
            .unwrap()
            .get(&message.version.to_string())
            .unwrap().clone();
        let host = IdMessageHandlerHost {
            kv: self.kv.clone(),
            engine: self.engine.clone(),
            id_linker: self.id_linker.clone(),
            id_components: self.id_components.clone(),
        };
        let mut store = wasmtime::Store::new(&self.engine, IdMessageHandlerState { host });
        let (idp2p, _) = Idp2pP2p::instantiate(&mut store, &component, &self.p2p_linker)?;
        let _ = idp2p
            .interface0
            .call_handle_message(store, IdMessageKind::Gossip, b"")?;
        Ok(())
    }

    pub(crate) async fn run(mut self) {
        loop {
            tokio::select! {
                cmd = self.handler_cmd_receiver.next() => match cmd {
                    Some(cmd) => todo!(),//self.handle_command(cmd).await.unwrap(),
                    None =>  return,
                },
            }
        }
    }
}

impl<S: KvStore> id_query::Host for IdMessageHandlerHost<S> {
    fn get(&mut self, id: String) -> Result<Option<Vec<u8>>, String> {
        todo!()
    }

    fn verify_inception(&mut self, version: u64, inception: Vec<u8>) -> Result<Vec<u8>, String> {
        todo!()
    }

    fn verify_event(
        &mut self,
        version: u64,
        view: Vec<u8>,
        event: Vec<u8>,
    ) -> Result<Vec<u8>, String> {
        todo!()
    }
}
