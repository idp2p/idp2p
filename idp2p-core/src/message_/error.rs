use idp2p_common::multi::error::Idp2pMultiError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum IdMessageError {
    #[error(transparent)]
    StdError(#[from] std::io::Error),
    #[error(transparent)]
    Idp2pMultiError(#[from] Idp2pMultiError),
    #[error("Other")]
    Other,
}
