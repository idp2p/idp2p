use idp2p_common::{
    multi::{ keypair::Idp2pKeypair, id::Idp2pCodec},
};

use crate::identity::state::IdentityState;
use thiserror::Error;
pub mod codec;

#[derive(Error, Debug)]
pub enum IdMessageError {
    #[error(transparent)]
    StdError(#[from] std::io::Error),
    #[error("Other")]
    Other
}

#[derive(Debug, Clone, PartialEq)]
pub struct IdMessage {
    id: Vec<u8>,
    body: Vec<u8>
}

pub struct IdMessageContent{
    from: Vec<u8>,
    to: Vec<u8>,
    created_at: i64,
    body: Vec<u8>
}

pub struct CreateIdMessageInput{
    codec: Idp2pCodec,
    from: IdentityState,
    to: IdentityState,
    auth_keypair: Idp2pKeypair,
    agree_keypair: Idp2pKeypair,
}

pub struct VerifyIdMessageInput{
    codec: Idp2pCodec,
    from: IdentityState,
    to: IdentityState,
    agree_keypair: Idp2pKeypair,
}

impl IdMessage {
    pub fn new(input: CreateIdMessageInput) -> Result<Self, IdMessageError> {
        todo!()
    }

    pub fn from_bytes(bytes: &[u8]) -> Result<Self, IdMessageError> {
        todo!()
    }

    pub fn to_bytes(&self) -> Vec<u8>{
        todo!()
    }

    pub fn verify(&self, input: VerifyIdMessageInput) -> Result<IdMessageContent, IdMessageError> {
        todo!()
    }
}
