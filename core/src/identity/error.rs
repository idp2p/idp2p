use thiserror::Error;

#[derive(Error, Debug)]
pub enum IdentityError {
    #[error("Invalid id")]
    InvalidId,
    #[error("Invalid Create Key")]
    InvalidCreateKey,
    #[error("Invalid Revoke Key")]
    InvalidRevokeKey,
    #[error("Invalid previous")]
    InvalidPrevious,
    #[error("Invalid event signature")]
    InvalidEventSignature,
    #[error("Invalid signer")]
    InvalidSigner,
    #[error("Invalid next")]
    InvalidNext,
    #[error("Invalid protobuf")]
    InvalidProtobuf,
    #[error(transparent)]
    DecodeError(#[from] prost::DecodeError),
    #[error(transparent)]
    Idp2pMultiError(#[from] idp2p_common::multi::error::Idp2pMultiError),
    #[error("Other")]
    Other,
}
