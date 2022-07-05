use idp2p_common::multi::{
    key::Idp2pKey,
    key_digest::Idp2pKeyDigest,
};
use std::collections::HashMap;

use crate::error::Idp2pError;

#[derive(PartialEq, Debug, Clone)]
pub struct KeyState {
    pub id: Vec<u8>,
    pub valid_at: i64,
    pub expired_at: Option<i64>,
    pub key: Vec<u8>,
}

#[derive(PartialEq, Debug, Clone)]
pub struct AgreementKeyState {
    pub id: Vec<u8>,
    pub valid_at: i64,
    pub expired_at: Option<i64>,
    pub key: Vec<u8>,
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
    pub last_event_id: Vec<u8>,
    pub next_key_digest: Vec<u8>,
    pub recovery_key_digest: Vec<u8>,
    pub assertion_keys: Vec<KeyState>,
    pub authentication_keys: Vec<KeyState>,
    pub agreement_keys: Vec<AgreementKeyState>,
    pub proofs: HashMap<Vec<u8>, ProofState>,
}

impl IdentityState {
    pub fn next_signer_key(&self, signer_bytes: &[u8]) -> Result<Idp2pKey, Idp2pError> {
        let key_digest = Idp2pKeyDigest::from_bytes(&self.next_key_digest)?;
        Ok(key_digest.to_next_key(signer_bytes)?)
    }
    pub fn next_recovery_key(&self, signer_bytes: &[u8]) -> Result<Idp2pKey, Idp2pError> {
        let key_digest = Idp2pKeyDigest::from_bytes(&self.recovery_key_digest)?;
        Ok(key_digest.to_next_key(signer_bytes)?)
    }
    pub fn get_latest_auth_key(&self) -> Option<KeyState> {
        None
    }
    pub fn get_latest_agree_key(&self) -> Option<AgreementKeyState> {
        None
    }
}
pub trait IdentityStateEventHandler<T> {
    fn handle_event(&mut self, timestamp: i64, event: T) -> Result<(), Idp2pError>;
}