use wasmtime::component::bindgen;

pub mod entry;
pub mod handler;
pub mod store;

bindgen!({
    world:"idp2p-id",
    path:  "../id/wit/world.wit",
    additional_derives: [PartialEq, Eq, Hash, Clone, serde::Serialize, serde::Deserialize],
});