use idp2p_common::multi::keypair::Idp2pKeypair;

use self::{
    error::IdentityError,
    models::{ChangeType, IdEvent},
    state::IdentityState, handler::id_handler::ProtoIdentityHandler,
};
pub mod doc;
pub mod error;
pub mod models;
pub mod state;
pub mod handler;

pub struct CreateIdentityInput {
    // Next key digest(multikey digest)
    pub next_key_digest: Vec<u8>,
    // Recovery key digest(multikey digest)
    pub recovery_key_digest: Vec<u8>,
    pub events: Vec<IdEvent>,
}

#[derive(Debug)]
pub struct ChangeInput {
    pub next_key_digest: Vec<u8>,
    pub signer_keypair: Idp2pKeypair,
    pub change: ChangeType,
}

#[derive(PartialEq, Debug, Clone)]
pub struct Identity {
    // Bytes of inception id(idp2p multi id)
    pub id: Vec<u8>,
    // Microledger bytes(can be protobuf, json ... encoded)
    pub microledger: Vec<u8>,
}

pub trait IdentityHandler {
    fn new(&self, input: CreateIdentityInput) -> Result<Identity, IdentityError>;
    fn change(&self, did: &mut Identity, input: ChangeInput) -> Result<bool, IdentityError>;
    fn verify(
        &self,
        did: &Identity,
        prev: Option<&Identity>,
    ) -> Result<IdentityState, IdentityError>;
}

impl Identity{
    pub fn new(input: CreateIdentityInput) -> Result<Self, IdentityError> {
        ProtoIdentityHandler{}.new(input)
    }
    
    pub fn change(&mut self, input: ChangeInput) -> Result<bool, IdentityError> {
        ProtoIdentityHandler{}.change(self, input)
    }
    
    /// Verify an identity and get state of identity
    pub fn verify(&self, prev: Option<&Identity>) -> Result<IdentityState, IdentityError> {
        ProtoIdentityHandler{}.verify(self, prev)
    }
}