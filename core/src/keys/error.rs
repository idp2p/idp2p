use thiserror::Error;

#[derive(Error, Debug)]
pub enum MultiKeyError {
    #[error(transparent)]
    StdError(#[from] std::io::Error),
    #[error(transparent)]
    MultihashError(#[from] cid::multihash::Error),
    #[error(transparent)]
    VarintReadError(#[from] unsigned_varint::io::ReadError),
    #[error("InvalidKeyCode")]
    InvalidKeyCode,
}
