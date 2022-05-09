use std::collections::HashMap;

use idp2p_common::{
    agreement_key::Idp2pAgreementKey, anyhow::Result, id_proof::Idp2pProof, key::Idp2pKey,
    key_digest::Idp2pKeyDigest, secret::Idp2pSecret, hash::Idp2pHash,
};
use serde::{Deserialize, Serialize};

use super::did_doc::IdentityDocument;

pub trait IdentityBehaviour {
    type IdType;
    fn create(input: CreateIdentityInput) -> Result<Self::IdType>;
    fn verify(&self) -> Result<IdentityState>;
    fn recover(&mut self, signer: Idp2pSecret, rec_digest: Idp2pKeyDigest) -> Result<()>;
    fn add_events(&mut self, signer: Idp2pSecret, events: Vec<IdentityEvent>) -> Result<()>;
    fn to_document(&self) -> Result<IdentityDocument>;
}

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub enum IdentityEvent {
    SetProof { proof: Idp2pProof },
    SetAssertionKey { public: Idp2pKey },
    SetAuthenticationKey { public: Idp2pKey },
    SetAgreementKey { public: Idp2pAgreementKey },
}

#[derive(PartialEq, Debug, Clone)]
pub struct AssertionMethod {
    pub valid_at: i64,
    pub expired_at: Option<i64>,
    pub ver_method: Idp2pKey,
}

#[derive(PartialEq, Debug, Clone)]
pub struct IdentityState {
    pub event_id: Vec<u8>,
    pub next_key_digest: Idp2pKeyDigest,
    pub recovery_key_digest: Idp2pKeyDigest,
    pub assertion_keys: Vec<Idp2pKey>,
    pub authentication_key: Option<Idp2pKey>,
    pub agreement_key: Option<Idp2pAgreementKey>,
    pub proofs: HashMap<Vec<u8>, Vec<u8>>,
}

pub struct CreateIdentityInput {
    pub hash_alg: Idp2pHash,
    pub next_key_digest: Idp2pKeyDigest,
    pub recovery_key_digest: Idp2pKeyDigest,
    pub events: Vec<IdentityEvent>,
}
