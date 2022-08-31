use idp2p_common::multi::ledgerkey::{Idp2pLedgerPublicKey, Idp2pLedgerPublicDigest};
use std::collections::HashMap;

use crate::error::Idp2pError;

#[derive(PartialEq, Debug, Clone)]
pub struct AssertionPublicKeyState {
    pub id: Vec<u8>,
    pub valid_at: i64,
    pub expired_at: Option<i64>,
    pub key_bytes: Vec<u8>,
}


#[derive(PartialEq, Debug, Clone)]
pub struct AuthenticationPublicKeyState {
    pub id: Vec<u8>,
    pub valid_at: i64,
    pub expired_at: Option<i64>,
    pub key_bytes: Vec<u8>,
}

#[derive(PartialEq, Debug, Clone)]
pub struct AgreementPublicKeyState {
    pub id: Vec<u8>,
    pub valid_at: i64,
    pub expired_at: Option<i64>,
    pub key_bytes: Vec<u8>,
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
    pub assertion_keys: Vec<AssertionPublicKeyState>,
    pub authentication_keys: Vec<AuthenticationPublicKeyState>,
    pub agreement_keys: Vec<AgreementPublicKeyState>,
    pub proofs: HashMap<Vec<u8>, ProofState>,
}

impl IdentityState {
    pub fn next_signer_key(&self, signer_bytes: &[u8]) -> Result<Idp2pLedgerPublicKey, Idp2pError> {
        let key_digest = Idp2pLedgerPublicKey::from_multi_bytes(&self.next_key_digest)?;
        todo!()
        //Ok(key_digest.to_next_key(signer_bytes)?)
    }
    pub fn next_rec_key(&self, signer_bytes: &[u8]) -> Result<Idp2pLedgerPublicKey, Idp2pError> {
        todo!()
        //let key_digest = Idp2pLedgerPublicDigest::from_bytes(&self.recovery_key_digest)?;
        //Ok(key_digest.to_next_key(signer_bytes)?)
    }
    pub fn get_latest_auth_key(&self) -> Option<&AuthenticationPublicKeyState> {
        self.authentication_keys.last()
    }
    pub fn get_latest_agree_key(&self) -> Option<&AgreementPublicKeyState> {
        self.agreement_keys.last()
    }
}
pub trait IdentityStateEventHandler<T> {
    fn handle_event(&mut self, timestamp: i64, event: T) -> Result<(), Idp2pError>;
}
