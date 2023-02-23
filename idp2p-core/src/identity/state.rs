use std::collections::HashMap;

use idp2p_common::multi::ledgerkey::{Idp2pLedgerPublicDigest, Idp2pLedgerPublicKey};


#[derive(PartialEq, Debug, Clone)]
pub struct IdentityState {
    pub id: Vec<u8>,
    pub last_event_id: Vec<u8>,
    pub next_key_digest: Vec<u8>,
    pub recovery_key_digest: Vec<u8>,
    pub proofs: Vec<Vec<u8>>
}

pub trait IdentityStateEventMapper<T> {
    fn map_event(&mut self, timestamp: i64, event: T) -> Result<(), Idp2pError>;
}

impl IdentityState {
    pub fn next_signer_key(&self, signer_pk: &[u8]) -> Result<Idp2pLedgerPublicKey, Idp2pError> {
        let key_digest = Idp2pLedgerPublicDigest::from_multi_bytes(&self.next_key_digest)?;
        key_digest.ensure_public(signer_pk)?;
        Ok(Idp2pLedgerPublicKey::new(key_digest.code(), signer_pk)?)
    }
    pub fn next_rec_key(&self, signer_pk: &[u8]) -> Result<Idp2pLedgerPublicKey, Idp2pError> {
        let key_digest = Idp2pLedgerPublicDigest::from_multi_bytes(&self.recovery_key_digest)?;
        key_digest.ensure_public(signer_pk)?;
        Ok(Idp2pLedgerPublicKey::new(key_digest.code(), signer_pk)?)
    }
    pub fn get_latest_auth_key(&self) -> Option<&AuthenticationPublicKeyState> {
        self.authentication_keys.last()
    }
    pub fn get_latest_agree_key(&self) -> Option<&AgreementPublicKeyState> {
        self.agreement_keys.last()
    }
    pub fn get_auth_key_by_id(&self, kid: &[u8]) -> Option<&AuthenticationPublicKeyState> {
        self.authentication_keys.iter().find(|pk| pk.id == kid)
    }
    pub fn get_agree_key_by_id(&self, kid: &[u8]) -> Option<&AgreementPublicKeyState> {
        self.agreement_keys.iter().find(|pk| pk.id == kid)
    }
}
