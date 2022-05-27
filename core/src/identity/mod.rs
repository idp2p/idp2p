use crate::{
    keys::{agreement_key::Idp2pAgreementKey, key::Idp2pKey, key_digest::Idp2pKeyDigest},
    secret::Idp2pSecret,
};

use self::{error::IdentityError, state::IdentityState};

use std::collections::HashMap;

#[derive(PartialEq, Debug, Clone)]
pub struct IdEvents {
    pub assertion_key: Option<Idp2pKey>,
    pub authentication_key: Option<Idp2pKey>,
    pub agreement_key: Option<Idp2pAgreementKey>,
    pub proofs: HashMap<Vec<u8>, Vec<u8>>,
}

pub struct CreateIdentityInput {
    pub next_key_digest: Idp2pKeyDigest,
    pub recovery_key_digest: Idp2pKeyDigest,
    pub events: IdEvents,
}

pub struct ChangeInput {
    pub next_key_digest: Idp2pKeyDigest,
    pub recovery_key_digest: Idp2pKeyDigest,
    pub signer: Idp2pSecret,
    pub events: IdEvents,
}

pub struct RecoverInput {
    pub next_key_digest: Idp2pKeyDigest,
    pub recovery_key_digest: Idp2pKeyDigest,
    pub signer: Idp2pSecret,
}

#[derive(PartialEq, Debug, Clone)]
pub struct Identity {
    pub id: Vec<u8>,
    pub microledger: Vec<u8>,
}

pub trait IdentityBehaviour {
    fn new(input: CreateIdentityInput) -> Result<Identity, IdentityError> {
        todo!()
    }
    fn change(&mut self, input: ChangeInput) -> Result<(), IdentityError> {
        todo!()
    }
    fn recover(&mut self, input: RecoverInput) -> Result<(), IdentityError> {
        todo!()
    }
    fn verify(&self) -> Result<IdentityState, IdentityError> {
        todo!()
    }
}

pub mod error;
pub mod state;
pub mod doc;
pub mod protobuf;

