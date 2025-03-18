use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

use idp2p::p2p::{self, id_verifier};
use wasmtime::{
    component::{bindgen, Component, Linker},
    Config, Engine, Store,
};

bindgen!({
    world:"idp2p-id",
    path:  "./core/id/wit/",
    additional_derives: [PartialEq, Eq, Hash, Clone, serde::Serialize, serde::Deserialize],
});

bindgen!({
    world:"idp2p-p2p",
    path:  "./core/p2p/wit/",
    additional_derives: [PartialEq, Eq, Hash, Clone, serde::Serialize, serde::Deserialize],
});

struct HostComponent {
    runtime: Arc<WasmRuntime>,
}

struct P2pState {
    host: HostComponent,
}

struct IdState;

impl p2p::id_verifier::Host for HostComponent {
    #[doc = " Verifies an initial identity inception event."]
    fn verify_inception(
        &mut self,
        component: String,
        incepiton: Vec<u8>,
    ) -> Result<Vec<u8>, String> {
        todo!()
    }

    #[doc = " Verifies an identity update event against the existing identity state."]
    fn verify_event(
        &mut self,
        component: String,
        state: Vec<u8>,
        event: Vec<u8>,
    ) -> Result<Vec<u8>, String> {
        todo!()
    }
}

struct WasmRuntime {
    engine: Engine,
    id_linker: Linker<IdState>,
    p2p_linker: Linker<P2pState>,
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
        p2p::id_verifier::add_to_linker(&mut p2p_linker, |state: &mut P2pState| &mut state.host)?;

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

    pub fn handle_pubsub_message(&self, runtime: Arc<Self>, topic: &str, msg: &[u8]) -> anyhow::Result<()> {
        // Create a new store with the secondary result
        let mut store = Store::new(
            &self.engine,
            P2pState {
                host: HostComponent { runtime },
            },
        );
        let comp = self.p2p_components.lock().unwrap().get("k").unwrap().to_owned();
        // Instantiate and call the primary component
        let (p2p, _) =
            Idp2pP2p::instantiate(&mut store, &comp, &self.p2p_linker)?;
        Ok(p2p.interface0.call_handle_pubsub(&mut store, topic, msg).unwrap().unwrap())
    }

    fn verify_inception(&self, inception: &[u8]) -> anyhow::Result<()> {
        // Create a new store with the secondary result
        let mut store = Store::new(&self.engine, IdState);
        let comp = self.id_components.lock().unwrap().get("k").unwrap().to_owned();
        // Instantiate and call the primary component
        let (verifier, _) = Idp2pId::instantiate(
            &mut store,
            &comp,
            &self.id_linker,
        )?;
        verifier.call_verify_inception(&mut store, inception).unwrap().unwrap();
        Ok(())
    }
}
fn convert_to_component(bytes: &[u8]) -> Vec<u8> {
    wit_component::ComponentEncoder::default()
        .module(&bytes)
        .unwrap()
        .encode()
        .unwrap()
}
