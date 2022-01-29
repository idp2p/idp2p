
use thiserror::Error;

#[derive(Error, Debug)]
pub enum IdentityError {
    #[error("Invalid id")]
    InvalidId,
    #[error("Invalid ledger")]
    InvalidLedger,
    #[error("Invalid previous")]
    InvalidPrevious,
    #[error("Invalid event signature")]
    InvalidEventSignature,
    #[error("Invalid signer")]
    InvalidSigner,
    #[error("Invalid recovery")]
    InvalidDocumentDigest,
    #[error("Invalid next")]
    InvalidNext,
    #[error("Unknown")]
    Unknown,
}

pub type IdKeySecret = Vec<u8>;
pub type IdKey = Vec<u8>;
pub type IdKeyDigest = Vec<u8>;

pub mod did;
pub mod did_doc;
pub mod eventlog;
pub mod microledger;