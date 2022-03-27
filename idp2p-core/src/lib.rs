use idp2p_common::thiserror::Error;
use serde::{Deserialize, Serialize};

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

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub enum IdentityEvent {
    Created { id: String },
    Updated { id: String },
    Published { id: String },
    ReceivedJwm { id: String, jwm: String },
}

macro_rules! check {
    ($e: expr, $err: expr) => {{
        if !$e {
            return Err($err);
        }
    }};
}

pub mod did;
pub mod did_doc;
pub mod eventlog;
pub mod microledger;
pub mod message;
pub mod store;
