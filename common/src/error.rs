use thiserror::Error;

#[derive(Error, Debug)]
pub enum CommonError {
    #[error("Error")]
    DecodeError,
    #[error("Error")]
    EncodeError,
    #[error("Error")]
    InvalidPublicKey,
    #[error("Error")]
    InvalidSignature,
    #[error("Error")]
    SignatureVerifyError,
    #[error("Error")]
    Unknown
}