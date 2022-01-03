use crate::encode_vec;
use crate::eventlog::EventLogPayload;
use crate::eventlog::{EventLog, EventLogChange};
use crate::to_verification_publickey;
use crate::IdentityError;
use crate::ED25519;
use crate::{generate_cid, hash, IdKeyDigest};
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use chrono::prelude::*;

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub struct MicroLedgerState {
    pub event_id: String,
    pub next_key_digest: IdKeyDigest,
    pub recovery_key_digest: IdKeyDigest,
    pub doc_digest: Vec<u8>,
    pub proofs: HashMap<Vec<u8>, Vec<u8>>, // extract only current value
}

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub struct MicroLedgerInception {
    #[serde(rename = "keyType")]
    pub key_type: String,
    #[serde(with = "encode_vec", rename = "nextKeyDigest")]
    pub next_key_digest: IdKeyDigest,
    #[serde(with = "encode_vec", rename = "recoveryKeyDigest")]
    pub recovery_key_digest: IdKeyDigest,
}

impl MicroLedgerInception {
    pub fn get_id(&self) -> String {
        generate_cid(self)
    }
}

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub struct MicroLedger {
    pub inception: MicroLedgerInception,
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub events: Vec<EventLog>,
}

impl MicroLedger {
    pub fn new(next_key_digest: &[u8], recovery_key_digest: &[u8]) -> MicroLedger {
        let inception = MicroLedgerInception {
            key_type: ED25519.to_owned(),
            next_key_digest: next_key_digest.to_owned(),
            recovery_key_digest: recovery_key_digest.to_owned(),
        };
        MicroLedger {
            inception,
            events: vec![],
        }
    }

    pub fn save_event(&mut self, signer: &[u8], next: &[u8], change: EventLogChange) {
        let signer_publickey = to_verification_publickey(&signer);
        let previous = self.get_previous_id();
        let payload = EventLogPayload {
            previous: previous,
            signer_key: signer_publickey,
            next_key_digest: next.to_owned(),
            change: change,
            timestamp: Utc::now().timestamp()
        };
        let proof = payload.sign(&signer);
        let event_log = EventLog::new(payload, proof);
        self.events.push(event_log);
    }
    pub fn verify(&self, cid: &str) -> Result<MicroLedgerState, IdentityError> {
        let mut state = MicroLedgerState {
            event_id: self.inception.get_id(),
            next_key_digest: self.inception.next_key_digest.clone(),
            recovery_key_digest: self.inception.recovery_key_digest.clone(),
            doc_digest: vec![],
            proofs: HashMap::new(),
        };
        check!(cid == self.inception.get_id(), IdentityError::InvalidId);
        for event in &self.events {
            let previous_valid = event.payload.previous == state.event_id;
            check!(previous_valid, IdentityError::InvalidPrevious);
            let event_valid = event.verify(&event.payload.signer_key);
            check!(event_valid, IdentityError::InvalidEventSignature);
            let signer_digest = hash(&event.payload.signer_key);
            match &event.payload.change {
                EventLogChange::SetDocument(digest) => {
                    let signer_valid = state.next_key_digest == signer_digest;
                    check!(signer_valid, IdentityError::InvalidSigner);
                    state.doc_digest = digest.value.clone()
                }
                EventLogChange::SetProof(stmt) => {
                    let signer_valid = state.next_key_digest == signer_digest;
                    check!(signer_valid, IdentityError::InvalidSigner);
                    state.proofs.insert(stmt.key.clone(), stmt.value.clone());
                }
                EventLogChange::Recover(recovery) => {
                    let rec_valid = state.recovery_key_digest == signer_digest;
                    check!(rec_valid, IdentityError::InvalidSigner);
                    state.recovery_key_digest = recovery.recovery_key_digest.clone();
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
    use crate::eventlog::*;
    use crate::*;

    #[test]
    fn id_test() {
        let expected_id = "bagaaieravphdumkejbohc7auy7c5od6dm6t2kw6ljhsoml3aoarzbhxxzeea";
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
    fn verify_invalid_previous_test() {
        let (mut ledger, secret) = create_microledger();
        let id = ledger.inception.get_id();
        let change = EventLogChange::SetDocument(DocumentDigest { value: vec![] });
        let signer = to_verification_publickey(&secret);
        ledger.save_event(&signer, &hash(&signer), change);
        ledger.events[0].payload.previous = "1".to_owned();
        let result = ledger.verify(&id);
        let is_err = matches!(result, Err(crate::IdentityError::InvalidPrevious));
        assert!(is_err, "{:?}", result);
    }

    #[test]
    fn verify_invalid_signature_test() {
        let (mut ledger, secret) = create_microledger();
        let id = ledger.inception.get_id();
        let change = EventLogChange::SetDocument(DocumentDigest { value: vec![] });
        let signer = to_verification_publickey(&secret);
        ledger.save_event(&signer, &hash(&signer), change);
        ledger.events[0].proof = vec![0; 64];
        let result = ledger.verify(&id);
        let is_err = matches!(result, Err(crate::IdentityError::InvalidEventSignature));
        assert!(is_err, "{:?}", result);
    }

    #[test]
    fn verify_set_doc_invalid_signer_test() {
        let (mut ledger, secret) = create_microledger();
        let id = ledger.inception.get_id();
        let change = EventLogChange::SetDocument(DocumentDigest { value: vec![] });
        let signer = to_verification_publickey(&secret);
        ledger.save_event(&signer, &hash(&signer), change);
        let new_secret = create_secret_key();
        let new_ed_key = to_verification_publickey(&new_secret);
        ledger.events[0].payload.signer_key = new_ed_key.clone();
        ledger.events[0].proof = ledger.events[0].payload.sign(&new_secret);
        let result = ledger.verify(&id);
        let is_err = matches!(result, Err(crate::IdentityError::InvalidSigner));
        assert!(is_err, "{:?}", result);
    }

    #[test]
    fn verify_set_proof_invalid_signer_test() {
        let (mut ledger, secret) = create_microledger();
        let id = ledger.inception.get_id();
        let change = EventLogChange::SetProof(ProofStatement {key: vec![], value: vec![] });
        let signer = to_verification_publickey(&secret);
        ledger.save_event(&signer, &hash(&signer), change);
        let new_secret = create_secret_key();
        let new_ed_key = to_verification_publickey(&new_secret);
        ledger.events[0].payload.signer_key = new_ed_key.clone();
        ledger.events[0].proof = ledger.events[0].payload.sign(&new_secret);
        let result = ledger.verify(&id);
        let is_err = matches!(result, Err(crate::IdentityError::InvalidSigner));
        assert!(is_err, "{:?}", result);
    }

    #[test]
    fn verify_recovery_invalid_signer_test() {
        let (mut ledger, secret) = create_microledger();
        let id = ledger.inception.get_id();
        let signer = to_verification_publickey(&secret);
        let rec = RecoverStatement {
            recovery_key_digest: hash(&signer),
        };
        let change = EventLogChange::Recover(rec);
        ledger.save_event(&signer, &hash(&signer), change);
        let new_secret = create_secret_key();
        let new_ed_key = to_verification_publickey(&new_secret);
        ledger.events[0].payload.signer_key = new_ed_key.clone();
        ledger.events[0].proof = ledger.events[0].payload.sign(&new_secret);
        let result = ledger.verify(&id);
        let is_err = matches!(result, Err(crate::IdentityError::InvalidSigner));
        assert!(is_err, "{:?}", result);
    }

    fn create_microledger() -> (MicroLedger, Vec<u8>) {
        let secret = "bd6yg2qeifnixj4x3z2fclp5wd3i6ysjlfkxewqqt2thie6lfnkma";
        let next_key = to_verification_publickey(&multibase::decode(secret).unwrap().1);
        let recovery_key = to_verification_publickey(&multibase::decode(secret).unwrap().1);
        let ledger = MicroLedger::new(&hash(&next_key), &hash(&recovery_key));
        (ledger, multibase::decode(secret).unwrap().1)
    }
}
