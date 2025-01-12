
use thiserror::Error;

#[derive(Error, Debug)]
pub enum HandleError {
    #[error("Identity not found: {0}")]
    IdNotFound(String),
    #[error("Peer not found: {0}")]
    PeerNotFound(String),
    #[error("Common error")]
    CommonError(#[from] idp2p_common::error::CommonError),
    #[error("Wasmtime error")]
    WasmtimeError(#[from] wasmtime::Error),
    #[error("IdError error")]
    IdError(#[from] idp2p_common::id::IdError),
    #[error("IdInceptionError error")]
    IdInceptionError(#[from] crate::IdInceptionError),
    #[error("IdEventError error")]
    IdEventError(#[from] crate::IdEventError),
    #[error("Unknown error")]
    Other(#[from] core::fmt::Error),
}