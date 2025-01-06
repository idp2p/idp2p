use wasmtime::component::bindgen;

pub mod message;
pub mod model;
pub mod handler;
pub mod verifier;
pub mod error;

bindgen!({
    world:"idp2p-id",
    path:  "../id/wit/",
    additional_derives: [PartialEq, Eq, Hash, Clone, serde::Serialize, serde::Deserialize],
});