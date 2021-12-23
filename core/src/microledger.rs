use crate::eventlog::{EventLog, EventLogChange};
use crate::IdentityError;
use crate::{generate_cid, hash, NextKey};
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub struct MicroLedgerState {
    pub event_id: String,
    pub next_key: NextKey,
    pub recovery_next_key: NextKey,
    pub proofs: HashMap<Vec<u8>, Vec<u8>>, // extract only current value
    pub doc_digest: Vec<u8>,
}

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub struct MicroLedgerInception {
    #[serde(rename = "signerNextKey")]
    pub signer_next_key: NextKey,
    #[serde(rename = "recoveryNextKey")]
    pub recovery_next_key: NextKey,
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
    pub fn new(public_key: &[u8], recovery_public_key: &[u8]) -> MicroLedger {
        let next_key = NextKey::from_public(public_key);
        let recovery_next_key = NextKey::from_public(recovery_public_key);
        let inception = MicroLedgerInception {
            signer_next_key: next_key,
            recovery_next_key: recovery_next_key,
        };
        MicroLedger {
            inception,
            events: vec![],
        }
    }

    pub fn verify(&self, cid: &str) -> Result<MicroLedgerState, IdentityError> {
        let mut state = MicroLedgerState {
            event_id: self.inception.get_id(),
            next_key: self.inception.signer_next_key.clone(),
            recovery_next_key: self.inception.recovery_next_key.clone(),
            proofs: HashMap::new(),
            doc_digest: vec![],
        };
        check!(cid == self.inception.get_id(), IdentityError::InvalidId);
        for event in &self.events {
            let previous_valid = event.payload.previous == state.event_id;
            check!(previous_valid, IdentityError::InvalidPrevious);
            let event_valid = event.verify(&event.payload.signer_public_key);
            check!(event_valid, IdentityError::InvalidEventSignature);
            let signer_digest = hash(&event.payload.signer_public_key);
            match &event.payload.change {
                EventLogChange::SetDocument(digest) => {
                    let signer_valid = state.next_key.value == signer_digest;
                    check!(signer_valid, IdentityError::InvalidSigner);
                    state.doc_digest = digest.value.clone()
                }
                EventLogChange::SetProof(stmt) => {
                    let signer_valid = state.next_key.value == signer_digest;
                    check!(signer_valid, IdentityError::InvalidSigner);
                    state.proofs.insert(stmt.key.clone(), stmt.value.clone());
                }
                EventLogChange::Recover(recovery) => {
                    let rec_valid = state.recovery_next_key.value == signer_digest;
                    check!(rec_valid, IdentityError::InvalidRecovery);
                    state.recovery_next_key = recovery.recovery_next_key.clone();
                }
            }
            state.next_key = event.payload.next_key.clone();
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
    fn new_test() {
        let inception_secret = create_secret_key();
        let ledger = MicroLedger::new(&inception_secret, &inception_secret);
        assert_eq!(ledger.events.len(), 0);
    }
    #[test]
    fn id_test() {
        let expected_id = "bagaaiera5ce3nckdmy5yd2hwzfpmcwnd2pldaqgbstgdrilhwaoanpzwsofa";
        let signer_secret = multibase::decode("bclc5pn2tfuhkqmupbr3lkyc5o4g4je6glfwkix6nrtf7hch7b3kq").unwrap().1;
        let recovery_secret = multibase::decode("bd6yg2qeifnixj4x3z2fclp5wd3i6ysjlfkxewqqt2thie6lfnkma").unwrap().1;
        let ledger = MicroLedger::new(&to_verification_publickey(&signer_secret), &to_verification_publickey(&recovery_secret));
        assert_eq!(ledger.inception.get_id(), expected_id);
    }
    
    #[test]
    fn verify_test() {
        let inception_signer_public = to_verification_publickey(&create_secret_key());
        let ledger = MicroLedger::new(&inception_signer_public, &inception_signer_public);
        let r = ledger.verify(&ledger.inception.get_id());
        assert!(r.is_ok());
    }

    #[test]
    fn verify_invalid_cid_test() {
        let public = to_verification_publickey(&create_secret_key());
        let ledger = MicroLedger::new(&public, &public);
        let id = format!("{}.", ledger.inception.get_id());
        let result = ledger.verify(&id);
        assert!(matches!(result, Err(crate::IdentityError::InvalidId)));
    }

    #[test]
    fn verify_invalid_previous_test() {
        let secret = create_secret_key();
        let public = to_verification_publickey(&secret);

        let mut ledger = MicroLedger::new(&public, &public);
        let payload = EventLogPayload {
            previous: "1".to_string(),
            signer_public_key: public.clone(),
            next_key : NextKey::from_public(&public),
            change: EventLogChange::SetDocument(DocumentDigest { value: vec![] }),
        };
        let proof = payload.sign(&secret);
        let log = EventLog::new(payload, proof);
        ledger.events.push(log);
        ledger.events[0].payload.previous = String::from("aa");
        let result = ledger.verify(&ledger.inception.get_id());
        assert!(matches!(result, Err(crate::IdentityError::InvalidPrevious)));
    }

    #[test]
    fn verify_invalid_signature_test() {
        let signer_secret = create_secret_key();
        let inception_public = to_verification_publickey(&signer_secret);
        let mut ledger = MicroLedger::new(&inception_public, &inception_public);
        let payload = EventLogPayload {
            previous: ledger.inception.get_id(),
            signer_public_key: inception_public.clone(),
            next_key: NextKey::from_public(&inception_public),
            change: EventLogChange::SetDocument(DocumentDigest { value: vec![] }),
        };
        let proof = payload.sign(&signer_secret);
        let log = EventLog::new(payload, proof);
        ledger.events.push(log);
        ledger.events[0].proof = vec![0; 64];
        let result = ledger.verify(&ledger.inception.get_id());
        assert!(matches!(
            result,
            Err(crate::IdentityError::InvalidEventSignature)
        ));
    }

    #[test]
    fn verify_invalid_signer_test() {
        let signer_secret = create_secret_key();
        let inception_public = to_verification_publickey(&signer_secret);
        let new_public = to_verification_publickey(&create_secret_key());
        let mut ledger = MicroLedger::new(&inception_public, &inception_public);
        let payload_rec = EventLogPayload {
            previous: ledger.inception.get_id(),
            signer_public_key: inception_public.clone(),
            next_key : NextKey::from_public(&new_public),
            change: EventLogChange::Recover(RecoverStatement {
                recovery_next_key: NextKey::from_public(&new_public),
            }),
        };
        let proof = payload_rec.sign(&signer_secret);
        let log_rec = EventLog::new(payload_rec, proof);
        ledger.events.push(log_rec.clone());
        let payload = EventLogPayload {
            previous: log_rec.get_id(),
            signer_public_key: inception_public.clone(),
            next_key : NextKey::from_public(&new_public),
            change: EventLogChange::SetDocument(DocumentDigest { value: vec![] }),
        };
        let proof = payload.sign(&signer_secret);
        let log = EventLog::new(payload, proof);
        ledger.events.push(log);
        let result = ledger.verify(&ledger.inception.get_id());
        assert!(matches!(result, Err(crate::IdentityError::InvalidSigner)));
    }

    #[test]
    fn verify_invalid_recovery_test() {
        let signer_secret = create_secret_key();
        let inception_public = to_verification_publickey(&signer_secret);
        let new_public = to_verification_publickey(&create_secret_key());
        let mut ledger = MicroLedger::new(&inception_public, &new_public);
        let payload_rec = EventLogPayload {
            previous: ledger.inception.get_id(),
            signer_public_key: inception_public.clone(),
            next_key: NextKey::from_public(&new_public),
            change: EventLogChange::Recover(RecoverStatement {
                recovery_next_key: NextKey::from_public(&new_public),
            }),
        };
        let proof = payload_rec.sign(&signer_secret);
        let log_rec = EventLog::new(payload_rec, proof);
        ledger.events.push(log_rec.clone());
        let result = ledger.verify(&ledger.inception.get_id());
        assert!(matches!(result, Err(crate::IdentityError::InvalidRecovery)));
    }
}
