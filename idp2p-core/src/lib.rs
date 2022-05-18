use idp2p_common::{thiserror::Error, regex};

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
    #[error("Invalid next")]
    InvalidNext,
    #[error("Invalid protobuf")]
    InvalidProtobuf,
    #[error("Unknown")]
    Unknown,
}

impl From<idp2p_common::anyhow::Error> for IdentityError {
    fn from(err: idp2p_common::anyhow::Error) -> Self {
        IdentityError::Unknown
    }
}

impl From<idp2p_common::cid::Error> for IdentityError {
    fn from(err: idp2p_common::cid::Error) -> Self {
        IdentityError::Unknown
    }
}

impl From<prost::DecodeError> for IdentityError {
    fn from(err: prost::DecodeError) -> Self {
        IdentityError::Unknown
    }
}

pub const ED25519_DID: &str = "Ed25519VerificationKey2020";
pub const X25519_DID: &str = "X25519KeyAgreementKey2020";
pub const ED25519_CODE: u64 = 0xed; 
pub const X25519_CODE: u64 = 0xec;
pub const SHA256_CODE: u64 = 0x12; 

pub enum Idp2pCodec {
    Protobuf = 0x50,
    Json = 0x0200,
}

pub fn is_idp2p(id: &str) -> bool {
    let re = regex::Regex::new(r"did:p2p:*").unwrap();
    re.is_match(id)
}

mod idp2p_proto {
    include!(concat!(env!("OUT_DIR"), "/idp2p.pb.rs"));
}
//pub mod identity;
//pub mod didcomm;
