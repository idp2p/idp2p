use std::collections::HashMap;

use idp2p_common::{
    agreement_key::Idp2pAgreementKey, anyhow::Result, encode_vec,
    key::Idp2pKey, key_digest::Idp2pKeyDigest, secret::Idp2pSecret, serde_with::skip_serializing_none,
};
use serde::{Deserialize, Serialize};

#[skip_serializing_none]
#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub struct VerificationMethod {
    pub id: String,
    pub controller: String,
    #[serde(rename = "type")]
    pub typ: String,
    #[serde(with = "encode_vec", rename = "publicKeyMultibase")]
    pub bytes: Vec<u8>,
}

pub trait IdentityBehaviour {
    fn create(input: CreateIdentityInput) -> Result<Self>
    where
        Self: Sized;
    fn verify(&self) -> Result<IdentityState>;
    fn recover(&mut self, signer: Idp2pSecret, next_key_digest: Idp2pKeyDigest, rec_digest: Idp2pKeyDigest) -> Result<()>;
    fn add_events(&mut self, signer: Idp2pSecret, next_key_digest: Idp2pKeyDigest, events: IdentityEvents) -> Result<()>;
}

#[derive(PartialEq, Debug, Clone)]
pub struct  IdentityEvents {
    pub proofs: HashMap<Vec<u8>, Vec<u8>>,
    pub authentication_key:  Idp2pKey,
    pub agreement_key:  Idp2pAgreementKey,
    pub assertion_keys: Vec<Idp2pKey>,
}

#[derive(PartialEq, Debug, Clone)]
pub struct AssertionKeyState {
    pub valid_at: i64,
    pub expired_at: Option<i64>,
    pub key: Idp2pKey,
}

#[derive(PartialEq, Debug, Clone)]
pub struct AuthenticationKeyState {
    pub valid_at: i64,
    pub expired_at: Option<i64>,
    pub key: Idp2pKey,
}

#[derive(PartialEq, Debug, Clone)]
pub struct AgreementKeyState {
    pub valid_at: i64,
    pub expired_at: Option<i64>,
    pub key: Idp2pAgreementKey,
}

#[derive(PartialEq, Debug, Clone)]
pub struct ProofState {
    pub valid_at: i64,
    pub expired_at: Option<i64>,
    pub value: Vec<u8>,
}

#[derive(PartialEq, Debug, Clone)]
pub struct  IdentityState {
    pub event_id: Vec<u8>,
    pub next_key_digest: Idp2pKeyDigest,
    pub recovery_key_digest: Idp2pKeyDigest,
    pub assertion_keys: Vec<AssertionKeyState>,
    pub authentication_keys: Vec<AuthenticationKeyState>,
    pub agreement_keys: Vec<AgreementKeyState>,
    pub proofs: HashMap<Vec<u8>, ProofState>,
}

pub struct CreateIdentityInput {
    pub next_key_digest: Idp2pKeyDigest,
    pub recovery_key_digest: Idp2pKeyDigest,
    pub events: IdentityEvents
}

#[skip_serializing_none]
#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub struct IdentityDocument {
    #[serde(rename = "@context")]
    pub context: Vec<String>,
    pub id: String,
    pub controller: String,
    #[serde(rename = "verificationMethod")]
    pub verification_method: Vec<VerificationMethod>,
    #[serde(rename = "assertionMethod")]
    pub assertion_method: Vec<String>,
    pub authentication: Vec<String>,
    #[serde(rename = "keyAgreement")]
    pub key_agreement: Vec<String>,
}

impl IdentityDocument {

    pub fn get_verification_method(&self, kid: &str) -> Option<VerificationMethod> {
        self.verification_method
            .clone()
            .into_iter()
            .find(|vm| vm.id == kid)
    }

    pub fn get_first_keyagreement(&self) -> String {
        self.key_agreement[0].clone()
    }
}