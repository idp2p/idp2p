use crate::multi::{agreement_key::Idp2pAgreementKey, key::Idp2pKey, key_digest::Idp2pKeyDigest, keypair::Idp2pKeypair};

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
    pub signer_keypair: Idp2pKeypair,
    pub events: IdEvents,
}

pub struct RecoverInput {
    pub next_key_digest: Idp2pKeyDigest,
    pub recovery_key_digest: Idp2pKeyDigest,
    pub signer_keypair: Idp2pKeypair,
}

#[derive(PartialEq, Debug, Clone)]
pub struct Identity {
    pub id: Vec<u8>,
    pub microledger: Vec<u8>,
}

pub trait IdentityBehaviour {
    fn new(input: CreateIdentityInput) -> Result<Identity, IdentityError>;
    fn change(&mut self, input: ChangeInput) -> Result<(), IdentityError>;
    fn recover(&mut self, input: RecoverInput) -> Result<(), IdentityError>;
    fn verify(&self) -> Result<IdentityState, IdentityError>;
}

pub mod doc;
pub mod error;
pub mod protobuf;
pub mod state;
