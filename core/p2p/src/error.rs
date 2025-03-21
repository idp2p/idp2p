
use thiserror::Error;

#[derive(Error, Debug)]
pub enum HandlePubsubMessageError {
    #[error("Identity not found: {0}")]
    IdNotFound(String),
    #[error("Peer not found: {0}")]
    PeerNotFound(String),
    #[error("Common error")]
    CommonError(#[from] idp2p_common::error::CommonError),
    #[error("Identifier error")]
    IdError(#[from] idp2p_common::identifier::IdentifierError),
    #[error("Unknown error")]
    Other(#[from] core::fmt::Error),
}