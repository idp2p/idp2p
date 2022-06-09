use std::collections::HashMap;

use idp2p_common::multi::{
    agreement_key::Idp2pAgreementKey, base::Idp2pBase, key::Idp2pKey, key_digest::Idp2pKeyDigest,
};

use super::{doc::{IdentityDocument, VerificationMethod}, error::IdentityError};

#[derive(PartialEq, Debug, Clone)]
pub struct KeyState {
    pub id: Vec<u8>,
    pub valid_at: i64,
    pub expired_at: Option<i64>,
    pub key: Idp2pKey,
}

#[derive(PartialEq, Debug, Clone)]
pub struct AgreementKeyState {
    pub id: Vec<u8>,
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
    pub last_event_id: Vec<u8>,
    pub next_key_digest: Idp2pKeyDigest,
    pub recovery_key_digest: Idp2pKeyDigest,
    pub assertion_keys: Vec<KeyState>,
    pub authentication_keys: Vec<KeyState>,
    pub agreement_keys: Vec<AgreementKeyState>,
    pub proofs: HashMap<Vec<u8>, ProofState>,
}

impl IdentityState{
    pub fn get_latest_auth_key(&self) -> Option<KeyState>{
        None
    }
    pub fn get_latest_agree_key(&self) -> Option<AgreementKeyState>{
        None
    }
}
pub trait IdentityStateEventHandler<T> {
    fn handle_event(&mut self, timestamp: i64, event: T) -> Result<(), IdentityError>;
}

impl Into<IdentityDocument> for IdentityState {
    fn into(self) -> IdentityDocument {
        let id_str = format!("did:p2p:{}", Idp2pBase::default().encode(&self.id));
        let mut doc = IdentityDocument {
            context: vec![
                "https://www.w3.org/ns/did/v1".to_string(),
                "https://w3id.org/security/suites/ed25519-2020/v1".to_string(),
                "https://w3id.org/security/suites/x25519-2020/v1".to_string(),
            ],
            id: id_str.clone(),
            controller: id_str.clone(),
            verification_method: vec![],
            assertion_method:  vec![],
            authentication:  vec![],
            key_agreement:  vec![],
        };
        for assertion_key in self.assertion_keys {
            let kid = Idp2pBase::default().encode(assertion_key.id);
            let assertion_id = format!("{id_str}#{kid}");
            let ver_method = VerificationMethod{
                id: kid,
                controller: id_str.clone(),
                typ: assertion_key.key.did_code(),
                bytes: assertion_key.key.to_bytes(),
            };
            doc.verification_method.push(ver_method);
            doc.assertion_method.push(assertion_id);
        }

        for authentication_key in self.authentication_keys {
            let kid = Idp2pBase::default().encode(authentication_key.id);
            let authentication_id = format!("{id_str}#{kid}");
            let ver_method = VerificationMethod{
                id: kid,
                controller: id_str.clone(),
                typ: authentication_key.key.did_code(),
                bytes: authentication_key.key.to_bytes(),
            };
            doc.verification_method.push(ver_method);
            doc.authentication.push(authentication_id);
        }
        for agreement_key in self.agreement_keys {
            let kid = Idp2pBase::default().encode(agreement_key.id);
            let agreement_id = format!("{id_str}#{kid}");
            let ver_method = VerificationMethod{
                id: kid,
                controller: id_str.clone(),
                typ: agreement_key.key.did_scheme(),
                bytes: agreement_key.key.to_bytes(),
            };
            doc.verification_method.push(ver_method);
            doc.key_agreement.push(agreement_id);
        }
        doc
    }
}
