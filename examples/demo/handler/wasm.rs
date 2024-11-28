use std::collections::HashMap;

use wasmtime::{component::Component, Engine};

pub struct IdWasmHandler {
    engine: Engine,
    id_components: HashMap<String, Component>,
    p2p_components: HashMap<String, Component>,
}

