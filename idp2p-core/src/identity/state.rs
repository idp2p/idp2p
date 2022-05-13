use std::collections::HashMap;

use idp2p_common::{
    agreement_key::Idp2pAgreementKey, anyhow::Result, key::Idp2pKey, key_digest::Idp2pKeyDigest,
};

use super::doc::IdentityDocument;

#[derive(PartialEq, Debug, Clone)]
pub struct KeyState {
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
pub struct IdentityState {
    pub id: Vec<u8>,
    pub event_id: Vec<u8>,
    pub next_key_digest: Idp2pKeyDigest,
    pub recovery_key_digest: Idp2pKeyDigest,
    pub assertion_keys: Vec<KeyState>,
    pub authentication_keys: Vec<KeyState>,
    pub agreement_keys: Vec<AgreementKeyState>,
    pub proofs: HashMap<Vec<u8>, ProofState>,
}

pub trait IdentityStateEventHandler<T> {
    fn handle_event(&mut self, timestamp: i64, event: T) -> Result<()>;
}

impl Into<IdentityDocument> for IdentityState {
    fn into(self) -> IdentityDocument {
        for assetion_key in self.assertion_keys {}
        let doc = IdentityDocument {
            context: vec![
                "https://www.w3.org/ns/did/v1".to_string(),
                "https://w3id.org/security/suites/ed25519-2020/v1".to_string(),
                "https://w3id.org/security/suites/x25519-2020/v1".to_string(),
            ],
            id: format!("did:p2p:{}", idp2p_common::encode(&self.id)),
            controller: format!("did:p2p:{}", idp2p_common::encode(&self.id)),
            verification_method: todo!(),
            assertion_method: todo!(),
            authentication: todo!(),
            key_agreement: todo!(),
        };
        todo!()
    }
}
