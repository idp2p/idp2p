use idp2p_common::{
    cid::Cid,
    multi::{
        error::Idp2pMultiError,
        id::{Idp2pCid, Idp2pCodec},
        keypair::Idp2pKeypair,
    },
};

use crate::{identity::state::IdentityState, idp2p_proto};
use prost::Message;
use thiserror::Error;
pub mod codec;
pub mod protobuf;

#[derive(Error, Debug)]
pub enum IdMessageError {
    #[error(transparent)]
    StdError(#[from] std::io::Error),
    #[error(transparent)]
    DecodeError(#[from] prost::DecodeError),
    #[error(transparent)]
    Idp2pMultiError(#[from] Idp2pMultiError),
    #[error("Other")]
    Other,
}

#[derive(Debug, Clone, PartialEq)]
pub struct IdMessage {
    id: Vec<u8>,
    body: Vec<u8>,
}

pub struct IdMessageContent {
    from: Vec<u8>,
    to: Vec<u8>,
    created_at: i64,
    body: Vec<u8>,
}

pub struct CreateIdMessageInput {
    codec: Idp2pCodec,
    from: IdentityState,
    to: IdentityState,
    auth_keypair: Idp2pKeypair,
    body: Vec<u8>,
}

pub struct VerifyIdMessageInput {
    from: IdentityState,
    to: IdentityState,
    agree_keypair: Idp2pKeypair,
}

impl IdMessage {
    pub fn new(input: CreateIdMessageInput) -> Result<Self, IdMessageError> {
        match input.codec {
            Idp2pCodec::Protobuf => {
                let body = protobuf::factory::new(input)?;
                Ok(IdMessage {
                    id: Cid::new_cid(Idp2pCodec::Protobuf, &body).to_bytes(),
                    body: body,
                })
            }
            Idp2pCodec::Json => todo!(),
        }
    }

    pub fn from_bytes(bytes: &[u8]) -> Result<Self, IdMessageError> {
        let msg = idp2p_proto::IdMessage::decode(bytes)?;
        Ok(IdMessage {
            id: msg.id,
            body: msg.content,
        })
    }

    pub fn verify(&self, input: VerifyIdMessageInput) -> Result<IdMessageContent, IdMessageError> {
        todo!()
    }
}

impl Into<idp2p_proto::IdMessage> for IdMessage {
    fn into(self) -> idp2p_proto::IdMessage {
        idp2p_proto::IdMessage {
            id: self.id,
            content: self.body,
        }
    }
}
