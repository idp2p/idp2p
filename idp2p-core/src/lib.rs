use idp2p_common::{anyhow::Result, encode_vec, thiserror::Error};
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
    Connected { id: String },
    PostHandled { id: String },
    GetHandled { id: String },
    JwmCreated { jwm: String },
    JwmReceived { jwm: String },
}

pub trait IdPersister {
    fn get(&self) -> Result<String>;
    fn persist(&self, s: &str) -> Result<()>;
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
pub mod message;
pub mod microledger;
pub mod store;
