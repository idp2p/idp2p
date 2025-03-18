use idp2p_common::{error::CommonError, identifier::IdentifierError};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum IdInceptionError {
    #[error("Invalid timestamp")]
    InvalidTimestamp,
    #[error("Invalid id: {0}")]
    InvalidId(String),
    #[error("Invalid payload")]
    InvalidPayload,
    #[error("Payload and id do not match")]
    PayloadAndIdNotMatch,
    #[error("Threshold not match")]
    ThresholdNotMatch,
    #[error("Next threshold not match")]
    NextThresholdNotMatch,
    #[error("Invalid signer: {0}")]
    InvalidSigner(String),
    #[error("Dublicate signer: {0}")]
    DublicateSigner(String),
    #[error("Invalid signer kind: {0}")]
    InvalidSignerKind(String),
    #[error("Invalid next signer: {0}")]
    InvalidNextSigner(String),
    #[error("Dublicate next signer: {0}")]
    DublicateNextSigner(String),
    #[error("Invalid next signer kind: {0}")]
    InvalidNextSignerKind(String),
    #[error("Invalid claim: {0}")]
    InvalidClaim(String),
    #[error("Other error: {0}")]
    Other(String),
    #[error("Common error:\n {0}")]
    CommonError(#[from] CommonError),
    #[error("Common error:\n {0}")]
    IdentifierError(#[from] IdentifierError),
}