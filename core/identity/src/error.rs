use idp2p_common::multi::error::Idp2pMultiError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum Idp2pError {
    #[error("Invalid id")]
    InvalidId,
    #[error("Invalid Create Key")]
    InvalidCreateKey,
    #[error("Invalid Revoke Key")]
    InvalidRevokeKey,
    #[error("Invalid previous identity")]
    InvalidPrevious,
    #[error("Invalid previous event log")]
    InvalidPreviousEventLog,
    #[error("Invalid event signature")]
    InvalidEventSignature,
    #[error("Invalid signer")]
    InvalidSigner,
    #[error("Invalid next")]
    InvalidNext,
    #[error("Required field should not be empty. Field name: {0}")]
    RequiredField(String),
    #[error(transparent)]
    DecodeError(#[from] prost::DecodeError),
    #[error(transparent)]
    MultihashError(#[from] idp2p_common::multihash::Error),
    #[error(transparent)]
    Idp2pMultiError(#[from] Idp2pMultiError),
    #[error("Other")]
    Other,
}
