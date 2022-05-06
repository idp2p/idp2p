//! ## MicroLedgerInception
//! key_type: Type of keys
//! master_key_digest: Master public key digest
//! next_key_digest: Next public key digest

use super::{
    eventlog::{EventLog, EventLogChange, EventLogPayload}
};
use crate::IdentityError;
use idp2p_common::{
    anyhow::Result, chrono::prelude::*, encode, encode_vec, generate_json_cid, hash, IdKeyDigest,
    IDP2P_ED25519, Idp2pCodec,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub struct AssertionMethod {
    pub valid_at: i64,
    pub expired_at: Option<i64>,
    pub ver_method: VerificationMethod,
}

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub struct MicroLedgerState {
    pub event_id: String,
    #[serde(with = "encode_vec")]
    pub next_key_digest: IdKeyDigest,
    #[serde(with = "encode_vec")]
    pub recovery_key_digest: IdKeyDigest,
    pub assertion_keys: Vec<AssertionMethod>,
    pub authentication_key: Option<VerificationMethod>,
    pub agreement_key: Option<VerificationMethod>,
    pub proofs: HashMap<String, String>,
}

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub struct MicroLedgerInception {
    #[serde(rename = "keyType")]
    pub key_type: String,
    #[serde(with = "encode_vec", rename = "recoveryKeyDigest")]
    pub recovery_key_digest: Vec<u8>,
    #[serde(with = "encode_vec", rename = "nextKeyDigest")]
    pub next_key_digest: IdKeyDigest,
}

impl MicroLedgerInception {
    pub fn get_id(&self) -> String {
        generate_json_cid(self).unwrap()
    }
}

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub struct MicroLedger {
    pub inception: MicroLedgerInception,
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub events: Vec<EventLog>,
}

impl MicroLedger {
    pub fn new(recovery_key_digest: &[u8], next_key_digest: &[u8]) -> Self {
        let inception = MicroLedgerInception {
            key_type: IDP2P_ED25519.to_owned(),
            recovery_key_digest: recovery_key_digest.to_owned(),
            next_key_digest: next_key_digest.to_owned(),
        };
        MicroLedger {
            inception,
            events: vec![],
        }
    }

    pub fn create_event(
        &self,
        signer_key: &[u8],
        next_digest: &[u8],
        change: Vec<EventLogChange>,
    ) -> EventLogPayload {
        let previous = self.get_previous_id();
        EventLogPayload {
            previous: previous,
            signer_key: signer_key.to_owned(),
            next_key_digest: next_digest.to_owned(),
            change: change,
            timestamp: Utc::now().timestamp(),
        }
    }

    pub fn save_event(&mut self, payload: EventLogPayload, proof: &[u8]) {
        let event_log = EventLog::new(payload, proof);
        self.events.push(event_log);
    }

    pub fn verify(&self, cid: &str) -> Result<MicroLedgerState, IdentityError> {
        let mut state = MicroLedgerState {
            event_id: self.inception.get_id(),
            recovery_key_digest: self.inception.recovery_key_digest.clone(),
            next_key_digest: self.inception.next_key_digest.clone(),
            assertion_keys: vec![],
            authentication_key: None,
            agreement_key: None,
            proofs: HashMap::new(),
        };
        check!(cid == self.inception.get_id(), IdentityError::InvalidId);
        for event in &self.events {
            let previous_valid = event.payload.previous == state.event_id;
            check!(previous_valid, IdentityError::InvalidPrevious);
            let event_valid = event.verify(&event.payload.signer_key);
            check!(event_valid, IdentityError::InvalidEventSignature);
            let signer_digest = hash(&event.payload.signer_key);
            check!(
                signer_digest == state.next_key_digest,
                IdentityError::InvalidSigner
            );
            for change in &event.payload.change {
                match &change {
                    EventLogChange::SetAssertionKey {
                        verification_method,
                    } => {
                        let previous_key = state.assertion_keys.last_mut();
                        if let Some(previous_key) = previous_key {
                            previous_key.expired_at = Some(event.payload.timestamp);
                        }
                        let assertion_method = AssertionMethod {
                            valid_at: event.payload.timestamp,
                            expired_at: None,
                            ver_method: verification_method.clone(),
                        };
                        state.assertion_keys.push(assertion_method);
                    }
                    EventLogChange::SetAuthenticationKey {
                        verification_method,
                    } => {
                        state.authentication_key = Some(verification_method.clone());
                    }
                    EventLogChange::SetAgreementKey {
                        verification_method,
                    } => {
                        state.agreement_key = Some(verification_method.clone());
                    }
                    EventLogChange::SetProof(stmt) => {
                        let key = encode(&stmt.key);
                        let value = encode(&stmt.value);
                        state.proofs.insert(key, value);
                    }
                }
            }
            state.next_key_digest = event.payload.next_key_digest.clone();
            state.event_id = event.get_id();
        }
        Ok(state)
    }

    pub fn get_previous_id(&self) -> String {
        let previous_id = if self.events.len() == 0 {
            self.inception.get_id()
        } else {
            let e = self.events.last().unwrap();
            e.get_id()
        };
        previous_id
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::json::did::eventlog::*;
    use idp2p_common::secret::EdSecret;
    use idp2p_common::ED25519;

    #[test]
    fn id_test() {
        let expected_id = "bagaaieraqun2pn4ycd3b4nq4ptyzfnxea4hohwlgd7vdu3cifiy2fowvvpuq";
        let ledger = create_microledger().0;
        assert_eq!(ledger.inception.get_id(), expected_id);
    }
    #[test]
    fn verify_test() {
        let ledger = create_microledger().0;
        let result = ledger.verify(&ledger.inception.get_id());
        assert!(result.is_ok(), "{:?}", result);
    }

    #[test]
    fn verify_invalid_id_test() {
        let ledger = create_microledger().0;
        let result = ledger.verify("1");
        let is_err = matches!(result, Err(crate::IdentityError::InvalidId));
        assert!(is_err, "{:?}", result);
    }

    #[test]
    fn verify_valid_test() {
        let (mut ledger, secret) = create_microledger();
        let id = ledger.inception.get_id();
        let set_proof = EventLogChange::SetProof(ProofStatement {
            key: vec![1],
            value: vec![1],
        });
        let ver_method = VerificationMethod {
            id: id.clone(),
            controller: id.clone(),
            typ: ED25519.to_string(),
            bytes: secret.to_publickey().to_vec(),
        };
        let set_assertion = EventLogChange::SetAssertionKey {
            verification_method: ver_method.clone(),
        };
        let set_authentication = EventLogChange::SetAuthenticationKey {
            verification_method: ver_method.clone(),
        };
        let set_agreement = EventLogChange::SetAgreementKey {
            verification_method: ver_method.clone(),
        };
        let change = vec![
            set_proof,
            set_assertion.clone(),
            set_authentication,
            set_agreement,
        ];
        let signer = secret.to_publickey();
        let payload = ledger.create_event(&signer, &secret.to_publickey_digest().unwrap(), change);
        let proof = secret.sign(&payload);
        ledger.save_event(payload, &proof);
        let change = vec![set_assertion];
        let payload = ledger.create_event(&signer, &signer, change);
        let proof = secret.sign(&payload);
        ledger.save_event(payload, &proof);
        let result = ledger.verify(&id);
        assert!(result.is_ok());
    }
    #[test]
    fn verify_invalid_previous_test() {
        let (mut ledger, secret) = create_microledger();
        let id = ledger.inception.get_id();
        let set_change = EventLogChange::SetProof(ProofStatement {
            key: vec![],
            value: vec![],
        });
        let change = vec![set_change];
        let signer = secret.to_publickey();
        let payload = ledger.create_event(&signer, &signer, change);
        let proof = secret.sign(&payload);
        ledger.save_event(payload, &proof);
        ledger.events[0].payload.previous = "1".to_owned();
        let result = ledger.verify(&id);
        let is_err = matches!(result, Err(crate::IdentityError::InvalidPrevious));
        assert!(is_err, "{:?}", result);
    }

    #[test]
    fn verify_invalid_signature_test() {
        let (mut ledger, secret) = create_microledger();
        let id = ledger.inception.get_id();
        let set_change = EventLogChange::SetProof(ProofStatement {
            key: vec![],
            value: vec![],
        });
        let change = vec![set_change];
        let signer = secret.to_publickey();
        let payload = ledger.create_event(&signer, &signer, change);
        let proof = secret.sign(&payload);
        ledger.save_event(payload, &proof);
        ledger.events[0].proof = vec![0; 64];
        let result = ledger.verify(&id);
        let is_err = matches!(result, Err(crate::IdentityError::InvalidEventSignature));
        assert!(is_err, "{:?}", result);
    }

    #[test]
    fn verify_invalid_signer_test() {
        let (mut ledger, secret) = create_microledger();
        let id = ledger.inception.get_id();
        let set_change = EventLogChange::SetProof(ProofStatement {
            key: vec![],
            value: vec![],
        });
        let change = vec![set_change];
        let signer = secret.to_publickey();
        let payload = ledger.create_event(&signer, &signer, change);
        let proof = secret.sign(&payload);
        ledger.save_event(payload, &proof);
        let new_secret = EdSecret::new();
        let new_ed_key = new_secret.to_publickey();
        ledger.events[0].payload.signer_key = new_ed_key.to_vec();
        ledger.events[0].proof = new_secret.sign(&ledger.events[0].payload).to_vec();
        let result = ledger.verify(&id);
        let is_err = matches!(result, Err(crate::IdentityError::InvalidSigner));
        assert!(is_err, "{:?}", result);
    }

    fn create_microledger() -> (MicroLedger, idp2p_common::secret::EdSecret) {
        let secret_str = "bd6yg2qeifnixj4x3z2fclp5wd3i6ysjlfkxewqqt2thie6lfnkma";
        let secret = idp2p_common::secret::EdSecret::from_str(secret_str).unwrap();
        let d = secret.to_publickey_digest().unwrap();
        let ledger = MicroLedger::new(&d, &d);
        (ledger, secret)
    }
}
