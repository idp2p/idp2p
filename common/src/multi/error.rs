use thiserror::Error;

#[derive(Error, Debug)]
pub enum Idp2pMultiError {
    #[error(transparent)]
    StdError(#[from] std::io::Error),
    #[error(transparent)]
    Ed25519Error(#[from] ed25519_dalek::ed25519::Error),
    #[error(transparent)]
    MultihashError(#[from] cid::multihash::Error),
    #[error(transparent)]
    MultibaseError(#[from] cid::multibase::Error),
    #[error(transparent)]
    VarintReadError(#[from] unsigned_varint::io::ReadError),
    #[error(transparent)]
    Infallible(#[from] std::convert::Infallible),
    #[error(transparent)]
    TryFromSliceError(#[from] std::array::TryFromSliceError),
    #[error("Invalid hash alg")]
    HashAlgError,
    #[error("InvalidKeyCode")]
    InvalidKeyCode,
    #[error("InvalidCid")]
    InvalidCid,
    #[error("InvalidDigest")]
    InvalidDigest,
    #[error("EncryptionError")]
    EncryptionError,
    #[error("DecryptionError")]
    DecryptionError,
}

