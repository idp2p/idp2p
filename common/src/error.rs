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
    #[error("Payload hash does not match the CID hash")]
    PayloadHashMismatch,
    #[error("Unsupported hash algorithm: {0}. Expected SHA2-256")]
    UnsupportedHashAlgorithm(u64),
    #[error("Unsupported codec: {0}. Expected raw")]
    UnsupportedCodec(u64),
    #[error("Invalid CID: {0}")]
    Multihash(#[from] cid::multihash::Error),
    #[error("Formatting error:\n {0}")]
    Other(#[from] core::fmt::Error),
}
