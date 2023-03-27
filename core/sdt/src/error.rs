use thiserror::Error;

#[derive(Error, Debug)]
pub enum SdtError {
    #[error(transparent)]
    StdError(#[from] std::io::Error),
    #[error(transparent)]
    SerdeJsonError(#[from] serde_json::Error),
    #[error(
        "Proof doesn't match the computed proof. Proof is {expected}, computed proof is {actual}."
    )]
    VerificationError { expected: String, actual: String },
    #[error("{0}")]
    Other(String),
}
