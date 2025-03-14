
use alloc::string::String;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum CommonError {
    #[error("{0}")]
    DecodeError(String),
    #[error("Error")]
    EncodeError,
    #[error("Error")]
    InvalidPublicKey,
    #[error("Error")]
    InvalidSignature,
    #[error("Error")]
    SignatureVerifyError,
    #[error("Error")]
    MultihashError(#[from] multihash::Error),
    #[error("Error")]
    Other(#[from] core::fmt::Error),
}
