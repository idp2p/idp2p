use anyhow::Result;
use std::collections::HashMap;

use wasmtime::{
    component::{Component, Linker},
    Config, Engine, Store,
};

use crate::{model::IdVerifier, IdView, Idp2pId, PersistedIdEvent, PersistedIdInception};

pub struct IdVerifierImpl {
    engine: Engine,
    id_components: HashMap<u64, Component>,
}

impl IdVerifierImpl {
    pub fn new(components: HashMap<u64, Vec<u8>>) -> Result<Self> {
        let engine = Engine::new(Config::new().wasm_component_model(true))?;
        let mut id_components: HashMap<u64, Component> = HashMap::new();
        for (version, bytes) in components {
            id_components.insert(version, convert_to_component(&bytes));
        }
        let handler = Self {
            engine,
            id_components,
        };
        Ok(handler)
    }

    pub fn add_component(&mut self, version: u64, bytes: &[u8]) {
        self.id_components
            .insert(version, convert_to_component(&bytes));
    }

    fn get_component(&self, version: u64) -> Result<(Idp2pId, Store<()>)> {
        let mut store = Store::new(&self.engine, ());
        let component = self.id_components.get(&version).unwrap().clone();
        let (id, _) = Idp2pId::instantiate(&mut store, &component, &Linker::new(&self.engine))?;
        Ok((id, store))
    }
}

impl IdVerifier for IdVerifierImpl {
    async fn verify_inception(
        &self,
        version: u64,
        inception: &PersistedIdInception,
    ) -> Result<IdView> {
        let (verifier, mut store) = self.get_component(version)?;
        let view = verifier.call_verify_inception(&mut store, inception)??;
        Ok(view)
    }
    async fn verify_event(
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

fn convert_to_component(bytes: &[u8]) -> Component {
    todo!()
}
