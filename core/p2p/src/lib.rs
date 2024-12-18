use wasmtime::component::bindgen;

pub mod message;
pub mod model;
pub mod handler;
pub mod topic;
pub mod verifier;

bindgen!({
    world:"idp2p-id",
    path:  "../id/wit/world.wit",
    additional_derives: [PartialEq, Eq, Hash, Clone, serde::Serialize, serde::Deserialize],
});