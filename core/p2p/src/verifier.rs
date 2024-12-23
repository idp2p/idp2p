use anyhow::Result;
use std::{collections::HashMap, sync::Mutex};

use wasmtime::{
    component::{Component, Linker},
    Config, Engine, Store,
};

use crate::{model::IdVerifier, IdView, Idp2pId, PersistedIdEvent, PersistedIdInception};

pub struct IdVerifierImpl {
    engine: Engine,
    id_components: Mutex<HashMap<String, Component>>,
}

impl IdVerifierImpl {
    pub fn new(components: HashMap<String, Vec<u8>>) -> Result<Self> {
        let engine = Engine::new(Config::new().wasm_component_model(true))?;
        let id_components: HashMap<String, Component> = HashMap::new();

        let mut handler = Self {
            engine,
            id_components: Mutex::new(id_components),
        };
        for (version, bytes) in components {
            handler.add_component(&version, &bytes);
        }
        Ok(handler)
    }

    pub fn add_component(&mut self, version: &str, bytes: &[u8]) {
        let component =
            Component::from_binary(&self.engine, &convert_to_component(&bytes)).unwrap();
        self.id_components.lock().unwrap().insert(version.to_string(), component);
    }

    fn get_component(&self, version: &str) -> Result<(Idp2pId, Store<()>)> {
        let mut store = Store::new(&self.engine, ());
        let component = self.id_components.lock().unwrap().get(version).unwrap().clone();
        let (id, _) = Idp2pId::instantiate(&mut store, &component, &Linker::new(&self.engine))?;
        Ok((id, store))
    }
}

impl IdVerifier for IdVerifierImpl {
    async fn verify_inception(
        &self,
        version: &str,
        inception: &PersistedIdInception,
    ) -> Result<IdView> {
        let (verifier, mut store) = self.get_component(version)?;
        let view = verifier.call_verify_inception(&mut store, inception)??;
        Ok(view)
    }
    async fn verify_event(
        &self,
        version: &str,
        view: &IdView,
        event: &PersistedIdEvent,
    ) -> Result<IdView> {
        let (verifier, mut store) = self.get_component(version)?;
        let view = verifier.call_verify_event(&mut store, view, event)??;
        Ok(view)
    }
}

fn convert_to_component(bytes: &[u8]) -> Vec<u8> {
    wit_component::ComponentEncoder::default()
        .module(&bytes)
        .unwrap()
        .encode()
        .unwrap()
}
