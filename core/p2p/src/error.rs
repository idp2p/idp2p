use alloc::string::String;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum HandlePubsubMessageError {
    #[error("Identity not found: {0}")]
    IdNotFound(String),
    #[error("Peer not found: {0}")]
    PeerNotFound(String),
    #[error("Common error")]
    CommonError(#[from] idp2p_common::error::CommonError),
    #[error("Host error")]
    HostError(#[from] crate::idp2p::p2p::types::P2pError),
    #[error("Identifier error")]
    IdError(#[from] idp2p_common::identifier::IdentifierError),
    #[error("Unknown error")]
    Other(#[from] core::fmt::Error),
}

#[derive(Error, Debug)]
pub enum HandleRequestError {
    #[error("Common error")]
    CommonError(#[from] idp2p_common::error::CommonError),
    #[error("Identifier error")]
    IdError(#[from] idp2p_common::identifier::IdentifierError),
    #[error("Unknown error")]
    Other(#[from] core::fmt::Error),
}