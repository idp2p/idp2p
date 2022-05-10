use std::collections::HashMap;

use idp2p_common::{
    agreement_key::Idp2pAgreementKey, anyhow::Result, hash::Idp2pHash, id_proof::Idp2pProof,
    key::Idp2pKey, key_digest::Idp2pKeyDigest, secret::Idp2pSecret,
};
use serde::{Deserialize, Serialize};

use super::did_doc::IdentityDocument;

pub trait IdentityBehaviour {
    fn create(input: CreateIdentityInput) -> Result<Self>
    where
        Self: Sized;
    fn verify(&self) -> Result<IdentityVerifyResult>;
    fn recover(&mut self, signer: Idp2pSecret, rec_digest: Idp2pKeyDigest) -> Result<()>;
    fn add_events(&mut self, signer: Idp2pSecret, events: Vec<IdentityEvent>) -> Result<()>;
    fn to_document(&self) -> Result<IdentityDocument>;
}

#[derive(PartialEq, Debug, Clone)]
pub enum IdentityEvent {
    SetProof { proof: Idp2pProof },
    SetAssertionKey { public: Idp2pKey },
    SetAuthenticationKey { public: Idp2pKey },
    SetAgreementKey { public: Idp2pAgreementKey },
}

#[derive(PartialEq, Debug, Clone)]
pub struct AssertionKey {
    pub valid_at: i64,
    pub expired_at: Option<i64>,
    pub ver_method: Idp2pKey,
}

#[derive(PartialEq, Debug, Clone)]
pub struct IdentityVerifyResult {
    pub id: Vec<u8>,
    pub next_key_digest: Idp2pKeyDigest,
    pub recovery_key_digest: Idp2pKeyDigest,
    pub assertion_keys: Vec<AssertionKey>,
    pub authentication_key: Option<Idp2pKey>,
    pub agreement_key: Option<Idp2pAgreementKey>,
    pub proofs: HashMap<Vec<u8>, Vec<u8>>,
}

#[derive(PartialEq, Debug, Clone)]
pub struct  IdentityState{
    pub event_id: Vec<u8>,
    pub next_key_digest: Idp2pKeyDigest,
    pub recovery_key_digest: Idp2pKeyDigest,
}

pub struct CreateIdentityInput {
    pub hash_alg: Idp2pHash,
    pub next_key_digest: Idp2pKeyDigest,
    pub recovery_key_digest: Idp2pKeyDigest,
    pub events: Vec<IdentityEvent>,
}
