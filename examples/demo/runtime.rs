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
    id_components:  Mutex<HashMap<String, Component>>,
    p2p_components:  Mutex<HashMap<String, Component>>,

}

impl WasmRuntime {
    // Initialize the runtime with both components
    pub fn new(id_comps: HashMap<String, Vec<u8>>, p2p_comps: HashMap<String, Vec<u8>>) -> anyhow::Result<Self> { // primary: Vec<u8>, secondary: Vec<u8>) -> anyhow::Result<Self> {
        // Create engine with component model enabled
        let engine = Engine::new(Config::new().wasm_component_model(true))?;

        // Create and set up linker
        let mut p2p_linker = Linker::new(&engine);
        p2p::id_verifier::add_to_linker(&mut p2p_linker, |state: &mut P2pState| {
            &mut state.host
        })?;

        let id_linker = Linker::new(&engine);

        let mut id_components: HashMap<String, Component> = HashMap::new();
        let mut p2p_components: HashMap<String, Component> = HashMap::new();

        for (id, bytes) in id_comps {
        }
        for (id, bytes) in p2p_comps {
        }
        Ok(WasmRuntime {
            engine,
            id_linker,
            p2p_linker,
            id_components: Mutex::new(id_components),
            p2p_components: Mutex::new(p2p_components),
        })
    }

    fn execute_primary(&self, runtime: Arc<Self>, input: &str) -> anyhow::Result<String> {
        // Create a new store with the secondary result
        let mut store = Store::new(
            &self.engine,
            PrimaryState {
                host: HostComponent { runtime },
            },
        );

        // Instantiate and call the primary component
        let (primary, _) =
            Primary::instantiate(&mut store, &self.primary_component, &self.primary_linker)?;
        primary.call_run(&mut store, input)
    }

    fn execute_secondary(&self, input: &str) -> anyhow::Result<String> {
        // Create a new store with the secondary result
        let mut store = Store::new(&self.engine, SecondaryState);

        // Instantiate and call the primary component
        let (secondary, _) = Secondary::instantiate(
            &mut store,
            &self.secondary_component,
            &self.secondary_linker,
        )?;
        secondary.call_verify(&mut store, input)
    }
}
fn convert_to_component(bytes: &[u8]) -> Vec<u8> {
    wit_component::ComponentEncoder::default()
        .module(&bytes)
        .unwrap()
        .encode()
        .unwrap()
}
