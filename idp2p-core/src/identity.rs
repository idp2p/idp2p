use idp2p_common::multi::{key_secret::Idp2pKeySecret, id::Idp2pId};

use crate::{error::Idp2pError, id_state::IdentityState, HandlerResolver};

#[derive(PartialEq, Debug, Clone)]
pub enum IdEvent {
    CreateAssertionKey { id: Vec<u8>, key: Vec<u8> },
    CreateAuthenticationKey { id: Vec<u8>, key: Vec<u8> },
    CreateAgreementKey { id: Vec<u8>, key: Vec<u8> },
    SetProof { key: Vec<u8>, value: Vec<u8> },
    RevokeAssertionKey(Vec<u8>),
    RevokeAuthenticationKey(Vec<u8>),
    RevokeAgreementKey(Vec<u8>),
}

#[derive(PartialEq, Debug, Clone)]
pub enum ChangeType {
    AddEvents { events: Vec<IdEvent> },
    Recover(Vec<u8>),
}

#[derive(PartialEq, Debug, Clone)]
pub struct CreateIdentityInput {
    pub timestamp: i64,
    // Next key digest(multikey digest)
    pub next_key_digest: Vec<u8>,
    // Recovery key digest(multikey digest)
    pub recovery_key_digest: Vec<u8>,
    pub events: Vec<IdEvent>,
}

#[derive(PartialEq, Debug, Clone)]
pub struct ChangeInput {
    pub next_key_digest: Vec<u8>,
    pub signer_secret: Idp2pKeySecret,
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
    fn new(&self, input: CreateIdentityInput) -> Result<Identity, Idp2pError>;
    fn change(&self, did: &mut Identity, input: ChangeInput) -> Result<bool, Idp2pError>;
    fn verify(
        &self,
        did: &Identity,
        prev: Option<&Identity>,
    ) -> Result<IdentityState, Idp2pError>;
}

impl Identity {
    pub fn change(&mut self, input: ChangeInput) -> Result<bool, Idp2pError> {
        let id = Idp2pId::from_bytes(&self.id)?;
        id.codec.resolve_id_handler().change(self, input)
    }

    /// Verify an identity and get state of identity
    pub fn verify(&self, prev: Option<&Identity>) -> Result<IdentityState, Idp2pError> {
        let id = Idp2pId::from_bytes(&self.id)?;
        id.codec.resolve_id_handler().verify(self, prev)
    }
}
