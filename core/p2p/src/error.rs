use alloc::string::String;
use thiserror::Error;

use crate::idp2p::p2p::types::P2pError;

#[derive(Error, Debug)]
pub enum HandlePubsubMessageError {
    #[error("Identity not found: {0}")]
    IdNotFound(String),
    #[error("Peer not found: {0}")]
    PeerNotFound(String),
    #[error("Provider not found: {0}")]
    ProviderNotFound(String),
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

impl From<HandlePubsubMessageError> for P2pError {
    fn from(value: HandlePubsubMessageError) -> Self {
        todo!()
    }
}