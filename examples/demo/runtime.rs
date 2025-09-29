use futures::channel::mpsc;
use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};
use wasmtime::{
    Config, Engine, Store,
    component::{Component, Linker, bindgen},
};

use crate::{
    network::IdNetworkCommand, runtime::verifier::id_verifier::idp2p::core::types::IdEventReceipt,
    store::InMemoryKvStore,
};

mod handler;
pub mod verifier;
struct HostComponent {
    runtime: Arc<WasmRuntime>,
    store: InMemoryKvStore,
    network_cmd_sender: mpsc::Sender<IdNetworkCommand>,
}

struct MessageState {
    host: HostComponent,
}

struct IdState;

struct WasmRuntime {
    engine: Engine,
    id_linker: Linker<IdState>,
    p2p_linker: Linker<MessageState>,
    id_components: Mutex<HashMap<String, Component>>,
    p2p_components: Mutex<HashMap<String, Component>>,
}

impl WasmRuntime {
    // Initialize the runtime with both components
    pub fn new(
        id_comps: HashMap<String, Vec<u8>>,
        p2p_comps: HashMap<String, Vec<u8>>,
    ) -> anyhow::Result<Self> {
        // primary: Vec<u8>, secondary: Vec<u8>) -> anyhow::Result<Self> {
        // Create engine with component model enabled
        let engine = Engine::new(Config::new().wasm_component_model(true))?;

        // Create and set up linker
        let mut p2p_linker = Linker::new(&engine);
        //p2p::p2p_host::add_to_linker(&mut p2p_linker, |state: &mut P2pState| &mut state.host)?;

        let id_linker = Linker::new(&engine);

        let mut id_components: HashMap<String, Component> = HashMap::new();
        let mut p2p_components: HashMap<String, Component> = HashMap::new();

        for (id, bytes) in id_comps {
            let component = Component::from_binary(&engine, &convert_to_component(&bytes)).unwrap();
            id_components.insert(id, component);
        }
        for (id, bytes) in p2p_comps {
            let component = Component::from_binary(&engine, &convert_to_component(&bytes)).unwrap();
            p2p_components.insert(id, component);
        }
        Ok(WasmRuntime {
            engine,
            id_linker,
            p2p_linker,
            id_components: Mutex::new(id_components),
            p2p_components: Mutex::new(p2p_components),
        })
    }

    pub fn handle_pubsub_message(
        &self,
        runtime: Arc<Self>,
        topic: &str,
        msg: &[u8],
    ) -> anyhow::Result<()> {
        // Create a new store with the secondary result
        let mut store = Store::new(
            &self.engine,
            MessageState {
                host: HostComponent {
                    runtime,
                    store: todo!(),
                    network_cmd_sender: todo!(),
                },
            },
        );
        let comp = self
            .p2p_components
            .lock()
            .unwrap()
            .get("k")
            .unwrap()
            .to_owned();
        // Instantiate and call the primary component
        let (handler, _) = handler::message_handler::Idp2pMessageHandler::instantiate(
            &mut store,
            &comp,
            &self.p2p_linker,
        )?;
        handler.call_handle(store, msg).unwrap();
        Ok(())
    }

    fn verify_inception(&self, inception: &IdEventReceipt) -> anyhow::Result<()> {
        // Create a new store with the secondary result
        let mut store = Store::new(&self.engine, IdState);
        let comp = self
            .id_components
            .lock()
            .unwrap()
            .get("k")
            .unwrap()
            .to_owned();
        // Instantiate and call the primary component
        let (verifier, _) = verifier::id_verifier::Idp2pIdVerifier::instantiate(
            &mut store,
            &comp,
            &self.id_linker,
        )?;
        verifier
            .idp2p_core_id_verifier()
            .call_verify_inception(&mut store, inception)
            .unwrap()
            .unwrap();
        Ok(())
    }
}

impl From<handler::message_handler::idp2p::core::types::IdProof>
    for verifier::id_verifier::idp2p::core::types::IdProof
{
    fn from(value: handler::message_handler::idp2p::core::types::IdProof) -> Self {
        Self {
            id: value.id,
            did: value.did,
            key_id: value.key_id,
            created: value.created,
            purpose: value.purpose,
            signature: value.signature,
        }
    }
}

impl From<handler::message_handler::idp2p::core::types::IdEventReceipt>
    for verifier::id_verifier::idp2p::core::types::IdEventReceipt
{
    fn from(value: handler::message_handler::idp2p::core::types::IdEventReceipt) -> Self {
        Self {
            id: value.id,
            version: value.version,
            created_at: value.created_at,
            payload: value.payload,
            proofs: value.proofs.into_iter().map(|p| p.into()).collect(),
        }
    }
}

fn convert_to_component(bytes: &[u8]) -> Vec<u8> {
    wit_component::ComponentEncoder::default()
        .module(&bytes)
        .unwrap()
        .encode()
        .unwrap()
}
