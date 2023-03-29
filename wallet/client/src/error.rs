use thiserror::Error;

#[derive(Error, Debug)]
pub enum WalletError {
    #[error(transparent)]
    StdError(#[from] std::io::Error),
    #[error("{0}")]
    Other(String),
}
