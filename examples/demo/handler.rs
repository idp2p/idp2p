use anyhow::Result;
use futures::{channel::mpsc, StreamExt};
use idp2p::p2p::id_query;
use std::{collections::HashMap, sync::Arc};
use wasmtime::{
    component::{bindgen, Component},
    Engine,
};

use crate::{
    command::{IdHandlerCommand, IdNetworkCommand},
    store::KvStore,
};
bindgen!({
    path: "core/p2p/wit",
    additional_derives: [PartialEq, Eq, Hash, Clone, serde::Serialize, serde::Deserialize],
});

#[derive(Clone)]
pub struct IdMessageHandler<S: KvStore> {
    store: Arc<S>,
    engine: Engine,
    id_components: HashMap<String, Component>,
    network_cmd_sender: mpsc::Sender<IdNetworkCommand>,
}

pub struct IdMessageEventLoop<S: KvStore> {
    store: Arc<S>,
    engine: Engine,
    p2p_components: HashMap<String, Component>,
    handler_cmd_receiver: mpsc::Receiver<IdHandlerCommand>,
}

impl<S: KvStore> IdMessageEventLoop<S> {
    pub fn new(
        store: Arc<S>,
        handler_cmd_receiver: mpsc::Receiver<IdHandlerCommand>,
    ) -> Result<Self> {
        todo!()
    }

    async fn handle_command(&mut self, cmd: IdHandlerCommand) -> Result<()> {
        let mut store = wasmtime::Store::new(&self.engine, String::new());
        Ok(())
    }

    pub(crate) async fn run(mut self) {
        loop {
            tokio::select! {
                cmd = self.handler_cmd_receiver.next() => match cmd {
                    Some(cmd) => self.handle_command(cmd).await.unwrap(),
                    // Command channel closed, thus shutting down the network event loop.
                    None =>  return,
                },
            }
        }
    }
}

impl<S: KvStore> id_query::Host for IdMessageHandler<S> {
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
