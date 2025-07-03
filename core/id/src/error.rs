use alloc::string::String;
use idp2p_common::error::CommonError;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum IdInceptionError {
    #[error("Invalid timestamp")]
    InvalidTimestamp,
    #[error("Invalid version")]
    UnsupportedVersion,
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
    #[error("Invalid CID: {0}")]
    Cid(#[from] cid::Error),
    #[error("Common error:\n {0}")]
    CommonError(#[from] CommonError)
}

#[derive(Debug, Error)]
pub enum IdEventError {
    #[error("Invalid timestamp")]
    InvalidTimestamp,
    #[error("Invalid version")]
    UnsupportedVersion,
    #[error("Invalid event id: {0}")]
    InvalidEventId(String),
    #[error("Invalid payload")]
    InvalidPayload,
    #[error("Payload and id do not match")]
    PayloadAndIdNotMatch,
    #[error("Previous not match")]
    PreviousNotMatch,
    #[error("Threshold not match")]
    ThresholdNotMatch,
    #[error("Next threshold not match")]
    NextThresholdNotMatch,
    #[error("Lack of minimum proofs")]
    LackOfMinProofs,
    #[error("Invalid proof: {kid}, {reason}")]
    InvalidProof { kid: String, reason: String },
    #[error("Invalid signer: {0}")]
    InvalidSigner(String),
    #[error("Invalid next signer: {0}")]
    InvalidNextSigner(String),
    #[error("Invalid claim: {0}")]
    InvalidClaim(String),
    #[error("Invalid delegation id: {0}")]
    InvalidDelegationId(String),
    #[error("Common error:\n {0}")]
    CommonError(#[from] CommonError),
}

impl IdEventError {
    pub fn invalid_proof(kid: &str, reason: &str) -> Self {
        Self::InvalidProof {
            kid: kid.to_owned(),
            reason: reason.to_owned(),
        }
    }
}
