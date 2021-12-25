use crate::encode_vec;
use crate::eventlog::{EventLog, EventLogChange};
use crate::IdentityError;
use crate::ED25519;
use crate::{generate_cid, hash, IdKeyDigest};
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

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
                    check!(rec_valid, IdentityError::InvalidRecovery);
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
        let secret = "bd6yg2qeifnixj4x3z2fclp5wd3i6ysjlfkxewqqt2thie6lfnkma";
        let next_key = to_verification_publickey(&multibase::decode(secret).unwrap().1);
        let recovery_key = to_verification_publickey(&multibase::decode(secret).unwrap().1);
        let ledger = MicroLedger::new(&hash(&next_key), &hash(&recovery_key));
        println!("{}", serde_json::to_string(&ledger).unwrap());
        assert_eq!(ledger.inception.get_id(), expected_id);
    }
    #[test]
    fn verify_test() {
        let secret = "bd6yg2qeifnixj4x3z2fclp5wd3i6ysjlfkxewqqt2thie6lfnkma";
        let next_key = to_verification_publickey(&multibase::decode(secret).unwrap().1);
        let recovery_key = to_verification_publickey(&multibase::decode(secret).unwrap().1);
        let ledger = MicroLedger::new(&next_key, &recovery_key);
        let result = ledger.verify(&ledger.inception.get_id());
        assert!(result.is_ok(), "{:?}", result);
    }

    #[test]
    fn verify_invalid_cid_test() {
        let secret = "bd6yg2qeifnixj4x3z2fclp5wd3i6ysjlfkxewqqt2thie6lfnkma";
        let next_key = to_verification_publickey(&multibase::decode(secret).unwrap().1);
        let recovery_key = to_verification_publickey(&multibase::decode(secret).unwrap().1);
        let ledger = MicroLedger::new(&next_key, &recovery_key);
        let id = format!("{}.", ledger.inception.get_id());
        let result = ledger.verify(&id);     
        let is_err = matches!(result, Err(crate::IdentityError::InvalidId));
        assert!(is_err, "{:?}", result);
    }

    #[test]
    fn verify_invalid_previous_test() {
        let secret = "bd6yg2qeifnixj4x3z2fclp5wd3i6ysjlfkxewqqt2thie6lfnkma";
        let next_key = to_verification_publickey(&multibase::decode(secret).unwrap().1);
        let recovery_key = to_verification_publickey(&multibase::decode(secret).unwrap().1);
        let mut ledger = MicroLedger::new(&next_key, &recovery_key);
        let payload = EventLogPayload {
            previous: "1".to_string(),
            signer_key: next_key.clone(),
            next_key_digest: hash(&next_key),
            change: EventLogChange::SetDocument(DocumentDigest { value: vec![] }),
        };
        let proof = payload.sign(&multibase::decode(secret).unwrap().1);
        let log = EventLog::new(payload, proof);
        ledger.events.push(log);
        ledger.events[0].payload.previous = String::from("aa");
        let result = ledger.verify(&ledger.inception.get_id());
        let is_err = matches!(result, Err(crate::IdentityError::InvalidPrevious));
        assert!(is_err, "{:?}", result);
    }

    #[test]
    fn verify_invalid_signature_test() {
        let signer_secret = create_secret_key();
        let inception_public = to_verification_publickey(&signer_secret);
        let mut ledger = MicroLedger::new(&inception_public, &inception_public);
        let payload = EventLogPayload {
            previous: ledger.inception.get_id(),
            signer_key: inception_public.clone(),
            next_key_digest: hash(&inception_public),
            change: EventLogChange::SetDocument(DocumentDigest { value: vec![] }),
        };
        let proof = payload.sign(&signer_secret);
        let log = EventLog::new(payload, proof);
        ledger.events.push(log);
        ledger.events[0].proof = vec![0; 64];
        let result = ledger.verify(&ledger.inception.get_id());
        let is_err = matches!(result, Err(crate::IdentityError::InvalidEventSignature));
        assert!(is_err, "{:?}", result);
    }

    #[test]
    fn verify_invalid_signer_test() {
        let secret = create_secret_key();
        let ed_key = to_verification_publickey(&secret);
        let new_ed_key = to_verification_publickey(&create_secret_key());
        let mut ledger = MicroLedger::new(&hash(&ed_key), &hash(&ed_key));
        let payload_rec = EventLogPayload {
            previous: ledger.inception.get_id(),
            signer_key: ed_key.clone(),
            next_key_digest: hash(&new_ed_key),
            change: EventLogChange::Recover(RecoverStatement {
                recovery_key_digest: hash(&new_ed_key),
            }),
        };
        let proof = payload_rec.sign(&secret);
        let log_rec = EventLog::new(payload_rec, proof);
        ledger.events.push(log_rec.clone());
        let payload = EventLogPayload {
            previous: log_rec.get_id(),
            signer_key: ed_key.clone(),
            next_key_digest: hash(&new_ed_key),
            change: EventLogChange::SetDocument(DocumentDigest { value: vec![] }),
        };
        let proof = payload.sign(&secret);
        let log = EventLog::new(payload, proof);
        ledger.events.push(log);
        let result = ledger.verify(&ledger.inception.get_id());
        let is_err = matches!(result, Err(crate::IdentityError::InvalidSigner));
        assert!(is_err, "{:?}", result);
    }

    #[test]
    fn verify_invalid_recovery_test() {
        let signer_secret = create_secret_key();
        let inception_public = to_verification_publickey(&signer_secret);
        let new_public = to_verification_publickey(&create_secret_key());
        let mut ledger = MicroLedger::new(&inception_public, &new_public);
        let payload_rec = EventLogPayload {
            previous: ledger.inception.get_id(),
            signer_key: inception_public.clone(),
            next_key_digest: hash(&new_public),
            change: EventLogChange::Recover(RecoverStatement {
                recovery_key_digest: hash(&new_public),
            }),
        };
        let proof = payload_rec.sign(&signer_secret);
        let log_rec = EventLog::new(payload_rec, proof);
        ledger.events.push(log_rec.clone());
        let result = ledger.verify(&ledger.inception.get_id());
        assert!(matches!(result, Err(crate::IdentityError::InvalidRecovery)));
    }
}
