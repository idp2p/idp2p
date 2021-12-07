use crate::eventlog::{EventLog, EventLogChange};
use crate::IdentityError;
use crate::{generate_cid, hash, RecoveryKey, SignerKey};
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub struct MicroLedgerState {
    pub current_event_id: String,
    pub current_signer_key: SignerKey,
    pub current_recovery_key: RecoveryKey,
    pub current_proofs: HashMap<Vec<u8>, Vec<u8>>, // extract only current value
    pub current_doc_digest: Vec<u8>,
}

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub struct MicroLedgerInception {
    pub signer_key: SignerKey,
    pub recovery_key: RecoveryKey,
}

impl MicroLedgerInception {
    pub fn get_id(&self) -> String {
        generate_cid(self)
    }
}

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub struct MicroLedger {
    pub id: String, // incepiton id
    pub inception: MicroLedgerInception,
    pub events: Vec<EventLog>,
}

impl MicroLedger {
    pub fn new(signer_key: Vec<u8>, recovery_key_digest: Vec<u8>) -> MicroLedger {
        let signer_key = SignerKey::new(signer_key);
        let recovery_key = RecoveryKey::new(recovery_key_digest);
        let inception = MicroLedgerInception {
            signer_key: signer_key,
            recovery_key: recovery_key,
        };
        let id = inception.get_id();
        MicroLedger {
            id,
            inception,
            events: vec![],
        }
    }

    pub fn verify(&self, cid: String) -> Result<MicroLedgerState, IdentityError> {
        let mut state = MicroLedgerState {
            current_event_id: self.inception.get_id(),
            current_signer_key: self.inception.signer_key.clone(),
            current_recovery_key: self.inception.recovery_key.clone(),
            current_proofs: HashMap::new(),
            current_doc_digest: vec![],
        };
        check!(cid == self.inception.get_id(), IdentityError::InvalidId);
        for event in &self.events {
            let previous_valid = event.payload.previous == state.current_event_id;
            check!(previous_valid, IdentityError::InvalidPrevious);
            let event_valid = event.verify(event.payload.signer_publickey.clone());
            check!(event_valid, IdentityError::InvalidEventSignature);
            match &event.payload.change {
                EventLogChange::SetDocument(digest) => {
                    let signer_valid =
                        state.current_signer_key.public == event.payload.signer_publickey.clone();
                    check!(signer_valid, IdentityError::InvalidSigner);
                    state.current_doc_digest = digest.value.clone()
                }
                EventLogChange::SetProof(stmt) => {
                    let signer_valid =
                        state.current_signer_key.public == event.payload.signer_publickey.clone();
                    check!(signer_valid, IdentityError::InvalidSigner);
                    state
                        .current_proofs
                        .insert(stmt.key.clone(), stmt.value.clone());
                }
                EventLogChange::Recover(recovery) => {
                    let recovery_key_digest = hash(&event.payload.signer_publickey.clone());
                    let rec_valid = recovery_key_digest == state.current_recovery_key.digest;
                    check!(rec_valid, IdentityError::InvalidRecovery);
                    state.current_signer_key = recovery.next_signer_key.clone();
                    state.current_recovery_key = recovery.next_recovery_key.clone();
                }
            }
            state.current_event_id = event.get_id();
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
        let inception_public = create_secret_key();
        let ledger = MicroLedger::new(inception_public.clone(), hash(&inception_public));
        assert_eq!(ledger.events.len(), 0);
        assert_eq!(ledger.inception.get_id(), ledger.id);
    }

    #[test]
    fn verify_test() {
        let inception_public = to_verification_publickey(create_secret_key());
        let ledger = MicroLedger::new(inception_public.clone(), hash(&inception_public));
        let r = ledger.verify(ledger.id.clone());
        assert!(r.is_ok());
    }

    #[test]
    fn verify_invalid_cid_test() {
        let inception_public = to_verification_publickey(create_secret_key());
        let mut ledger = MicroLedger::new(inception_public.clone(), hash(&inception_public));
        ledger.id = format!("{}.", ledger.id);
        let result = ledger.verify(ledger.id.clone());
        assert!(matches!(result, Err(crate::IdentityError::InvalidId)));
    }

    #[test]
    fn verify_invalid_previous_test() {
        let signer_secret = create_secret_key();
        let inception_public = to_verification_publickey(signer_secret.clone());
        let mut ledger = MicroLedger::new(inception_public.clone(), hash(&inception_public));
        let payload = EventLogPayload {
            previous: "1".to_string(),
            signer_publickey: inception_public.clone(),
            change: EventLogChange::SetDocument(DocumentDigest { value: vec![] }),
        };
        let log = EventLog::new(payload, signer_secret);
        ledger.events.push(log);
        ledger.events[0].payload.previous = String::from("aa");
        let result = ledger.verify(ledger.id.clone());
        assert!(matches!(result, Err(crate::IdentityError::InvalidPrevious)));
    }

    #[test]
    fn verify_invalid_signature_test() {
        let signer_secret = create_secret_key();
        let inception_public = to_verification_publickey(signer_secret.clone());
        let mut ledger = MicroLedger::new(inception_public.clone(), hash(&inception_public));
        let payload = EventLogPayload {
            previous: ledger.inception.get_id(),
            signer_publickey: inception_public.clone(),
            change: EventLogChange::SetDocument(DocumentDigest { value: vec![] }),
        };
        let log = EventLog::new(payload, signer_secret);
        ledger.events.push(log);
        ledger.events[0].proof = vec![0; 64];
        let result = ledger.verify(ledger.id.clone());
        assert!(matches!(result, Err(crate::IdentityError::InvalidPrevious)));
        /*match result {
            Ok(_) => println!("Success"),
            Err(error) => println!("Error: {:?}", error),
        };*/
    }

    #[test]
    fn verify_invalid_signer_test() {
        let signer_secret = create_secret_key();
        let inception_public = to_verification_publickey(signer_secret.clone());
        let new_public = to_verification_publickey(create_secret_key());
        let mut ledger = MicroLedger::new(inception_public.clone(), hash(&inception_public));
        let payload_rec = EventLogPayload {
            previous: ledger.inception.get_id(),
            signer_publickey: inception_public.clone(),
            change: EventLogChange::Recover(RecoverStatement {
                next_signer_key: SignerKey::new(new_public.clone()),
                next_recovery_key: RecoveryKey::new(hash(&new_public).to_vec()),
            }),
        };
        let log_rec = EventLog::new(payload_rec, signer_secret.clone());
        ledger.events.push(log_rec.clone());
        let payload = EventLogPayload {
            previous: log_rec.get_id(),
            signer_publickey: inception_public.clone(),
            change: EventLogChange::SetDocument(DocumentDigest { value: vec![] }),
        };
        let log = EventLog::new(payload, signer_secret);
        ledger.events.push(log);
        let result = ledger.verify(ledger.id.clone());
        assert!(matches!(result, Err(crate::IdentityError::InvalidSigner)));
    }

    #[test]
    fn verify_invalid_recovery_test() {
        let signer_secret = create_secret_key();
        let inception_public = to_verification_publickey(signer_secret.clone());
        let new_public = to_verification_publickey(create_secret_key());
        let mut ledger = MicroLedger::new(inception_public.clone(), hash(&new_public));
        let payload_rec = EventLogPayload {
            previous: ledger.inception.get_id(),
            signer_publickey: inception_public.clone(),
            change: EventLogChange::Recover(RecoverStatement {
                next_signer_key: SignerKey::new(new_public.clone()),
                next_recovery_key: RecoveryKey::new(hash(&new_public).to_vec()),
            }),
        };
        let log_rec = EventLog::new(payload_rec, signer_secret.clone());
        ledger.events.push(log_rec.clone());
        let result = ledger.verify(ledger.id.clone());
        assert!(matches!(result, Err(crate::IdentityError::InvalidRecovery)));
    }
}
