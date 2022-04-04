use idp2p_common::{encode_vec, thiserror::Error, anyhow::Result};
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
    Skipped { id: String },
    Publish { id: String },
}

pub trait IdPersister {
    fn get(&self) -> Result<String>;
    fn persist(&self, s: &str) -> Result<()>;
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct IdProfile {
    pub name: String,
    #[serde(with = "encode_vec")]
    pub photo: Vec<u8>,
}

impl IdProfile {
    pub fn new(name: &str, photo: &[u8]) -> Self {
        Self {
            name: name.to_owned(),
            photo: photo.to_owned(),
        }
    }
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
