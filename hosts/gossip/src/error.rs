use idp2p_common::multi::error::Idp2pMultiError;
use idp2p_core::error::Idp2pError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum Idp2pGossipError {
    #[error(transparent)]
    Idp2pError(#[from] Idp2pError),
    #[error("Required field should not be empty. Field name: {0}")]
    RequiredField(String),
    #[error(transparent)]
    Idp2pMultiError(#[from] Idp2pMultiError),
    #[error("Other")]
    Other,
}
