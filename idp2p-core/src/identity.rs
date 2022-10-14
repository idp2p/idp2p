use idp2p_common::multi::{
    id::{Idp2pCodec, Idp2pId},
};

use crate::error::Idp2pError;

use self::{models::{MutateInput, CreateInput}, state::IdentityState, codec::proto::ProtoIdentityCodec};
mod state;
mod models;
mod codec;

#[derive(PartialEq, Debug, Clone)]
pub struct Identity {
    // Bytes of inception id(idp2p multi id)
    pub id: Vec<u8>,
    // Microledger bytes(can be protobuf, json ... encoded)
    pub microledger: Vec<u8>,
}

pub trait IdentityCodec {
    fn new(&self, input: CreateInput) -> Result<Identity, Idp2pError>;
    fn mutate(&self, did: &mut Identity, input: MutateInput) -> Result<bool, Idp2pError>;
    fn verify(&self, did: &Identity, prev: Option<&Identity>) -> Result<IdentityState, Idp2pError>;
}

impl Identity {
    pub fn new_protobuf(input: CreateInput) -> Result<Identity, Idp2pError> {
        ProtoIdentityCodec.new(input)
    }

    pub fn mutate(&mut self, input: MutateInput) -> Result<bool, Idp2pError> {
        let id = Idp2pId::from_bytes(&self.id)?;
        match id.codec {
            Idp2pCodec::Protobuf => ProtoIdentityCodec.mutate(self, input),
            Idp2pCodec::Json => todo!(),
        }
    }

    /// Verify an identity and get state of identity
    pub fn verify(&self, prev: Option<&Identity>) -> Result<IdentityState, Idp2pError> {
        let id = Idp2pId::from_bytes(&self.id)?;
        match id.codec {
            Idp2pCodec::Protobuf => ProtoIdentityCodec.verify(self, prev),
            Idp2pCodec::Json => todo!(),
        }
    }
}
