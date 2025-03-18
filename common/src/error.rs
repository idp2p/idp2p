use alloc::string::String;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum CommonError {
    #[error("Decoding error: {0}")]
    DecodeError(String),
    #[error("Encoding error occurred")]
    EncodeError,
    #[error("Invalid public key provided")]
    InvalidPublicKey,
    #[error("Invalid signature provided")]
    InvalidSignature,
    #[error("Signature verification failed")]
    SignatureVerifyError,
    #[error("Invalid versioned message")]
    InvalidVersionedMessage,
    #[error("Multihash error:\n {0}")]
    MultihashError(#[from] multihash::Error),
    #[error("Formatting error:\n {0}")]
    Other(#[from] core::fmt::Error),
}
