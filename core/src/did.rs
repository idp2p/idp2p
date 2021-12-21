use crate::did_doc::IdDocument;
use crate::eventlog::DocumentDigest;
use crate::eventlog::EventLog;
use crate::eventlog::EventLogChange;
use crate::eventlog::EventLogPayload;
use crate::eventlog::ProofStatement;
use crate::eventlog::RecoverStatement;
use crate::microledger::MicroLedger;
use crate::*;
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::convert::TryInto;

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub struct Identity {
    pub id: String,
    pub microledger: MicroLedger,
    pub did_doc: IdDocument,
}

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub struct CreateIdentityResult {
    pub did: Identity,
    #[serde(with = "encode_me")]
    pub signer_secret: Vec<u8>,
    #[serde(with = "encode_me")]
    pub recovery_secret: Vec<u8>,
    #[serde(with = "encode_me")]
    pub assertion_secret: Vec<u8>,
    #[serde(with = "encode_me")]
    pub authentication_secret: Vec<u8>,
    #[serde(with = "encode_me")]
    pub keyagreement_secret: Vec<u8>,
}

pub enum IdentityEvent {
    SetProof {
        key: Vec<u8>,
        value: Vec<u8>,
        signer: Vec<u8>,
    },
    ChangeDoc {
        doc: IdDocument,
        signer: Vec<u8>,
    },
    Recover {
        new_signer: Vec<u8>,
        new_recovery: Vec<u8>,
        signer: Vec<u8>,
    },
}

impl Identity {
    pub fn new() -> CreateIdentityResult {
        let signer_secret = create_secret_key();
        let recovery_secret = create_secret_key();
        let assertion_secret = create_secret_key();
        let authentication_secret = create_secret_key();
        let key_agreement_secret = create_secret_key();
        Identity::new_with_secrets(
            signer_secret,
            recovery_secret,
            assertion_secret,
            authentication_secret,
            key_agreement_secret,
        )
    }

    pub fn new_with_secrets(
        signer_secret: Vec<u8>,
        rec_secret: Vec<u8>,
        assertion_secret: Vec<u8>,
        authentication_secret: Vec<u8>,
        keyagreement_secret: Vec<u8>,
    ) -> CreateIdentityResult {
        let rec_public = to_verification_publickey(rec_secret.clone());
        let signer_public = to_verification_publickey(signer_secret.clone());
        let recovery_public_bytes: [u8; 32] = rec_public.try_into().unwrap();
        let ledger = MicroLedger::new(signer_public, hash(&recovery_public_bytes));
        let id = ledger.inception.get_id();
        let doc_result = IdDocument::new_with_secrets(
            id.clone(),
            assertion_secret,
            authentication_secret,
            keyagreement_secret,
        );
        let mut did = Identity {
            id: id.clone(),
            microledger: ledger,
            did_doc: doc_result.doc.clone(),
        };
        did.set_doc_proof(doc_result.doc.clone(), signer_secret.clone());
        CreateIdentityResult {
            did: did,
            signer_secret: signer_secret,
            recovery_secret: rec_secret,
            assertion_secret: doc_result.assertion_secret,
            authentication_secret: doc_result.authentication_secret,
            keyagreement_secret: doc_result.keyagreement_secret,
        }
    }

    pub fn save_event(&mut self, e: IdentityEvent) {
        match e {
            IdentityEvent::SetProof { key, value, signer } => {
                let proof_stmt = ProofStatement {
                    key: key,
                    value: value,
                };
                let change = EventLogChange::SetProof(proof_stmt);
                let signer_publickey = to_verification_publickey(signer.clone());
                let payload = EventLogPayload {
                    previous: self.microledger.get_previous_id(),
                    change: change,
                    signer_publickey: signer_publickey,
                };
                let event_log = EventLog::new(payload, signer.clone());
                self.microledger.events.push(event_log);
            }
            IdentityEvent::Recover {
                new_signer,
                new_recovery,
                signer,
            } => {
                let change = EventLogChange::Recover(RecoverStatement {
                    next_recovery_key: IdKey::new(hash(&new_recovery)),
                });
                let signer_publickey = to_verification_publickey(signer.clone());
                let payload = EventLogPayload {
                    previous: self.microledger.get_previous_id(),
                    next_signer_key: IdKey::new(new_signer.clone()),
                    change: change,
                    signer_publickey: signer_publickey,
                };
                let event_log = EventLog::new(payload, signer.clone());
                self.microledger.events.push(event_log);
            }
            IdentityEvent::ChangeDoc { doc, signer } => {
                self.did_doc = doc.clone();
                self.set_doc_proof(doc.clone(), signer);
            }
        }
    }

    pub fn verify(&self) -> Result<bool, IdentityError> {
        let verified = self
            .microledger
            .verify(self.microledger.inception.get_id())?;
        let did_doc_digest = hash(serde_json::to_string(&self.did_doc).unwrap().as_bytes());
        check!(
            did_doc_digest == verified.current_doc_digest,
            IdentityError::InvalidDocumentDigest
        );
        Ok(true)
    }

    pub fn is_next(&self, new_did: Identity) -> Result<bool, IdentityError> {
        let mut candidate = self.clone();
        let last_id = candidate.microledger.events.last().unwrap().get_id();
        let mut is_last = false;
        for event in &new_did.microledger.events {
            if is_last {
                candidate.microledger.events.push(event.clone());
            }
            if event.get_id() == last_id {
                is_last = true;
            }
        }
        candidate.did_doc = new_did.did_doc.clone();
        let did_valid = candidate.get_digest() == new_did.get_digest();
        check!(did_valid, IdentityError::InvalidNext);
        candidate.verify()
    }

    pub fn get_digest(&self) -> String {
        let digest = hash(serde_json::to_string(&self).unwrap().as_bytes());
        crate::encode(digest)
    }

    fn set_doc_proof(&mut self, doc: IdDocument, secret_key: Vec<u8>) {
        let change = EventLogChange::SetDocument(DocumentDigest {
            value: doc.get_digest(),
        });
        let public_key = to_verification_publickey(secret_key.clone());
        let payload = EventLogPayload {
            previous: self.microledger.get_previous_id(),
            change: change,
            signer_publickey: public_key,
        };
        let event_log = EventLog::new(payload, secret_key.clone());
        self.microledger.events.push(event_log);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn create_with_secrets_test() {
        let result = create_did();
        let id = "bagaaierazg6rvoe5xoqcmbiz3qf2mztwl23g2vvmamotvm7rpv2fvgybo4qq";
        println!("{:?}", serde_json::to_string_pretty(&result.did).unwrap());
        assert_eq!(result.did.id, id);
    }

    #[test]
    fn set_doc_test() {
        let mut result = create_did();
        let old_doc_authentication = result.did.did_doc.authentication[0].clone();
        let new_doc = IdDocument::new(result.did.id.clone());
        result.did.save_event(IdentityEvent::ChangeDoc {
            doc: new_doc.doc,
            signer: result.signer_secret.clone(),
        });
        assert_eq!(result.did.microledger.events.len(), 2);
        assert_ne!(result.did.did_doc.authentication[0], old_doc_authentication);
    }

    #[test]
    fn is_next_ok_test() {
        let mut result = create_did();
        let current = result.did.clone();
        //result.did.set_doc(result.signer_secret.clone());
        let r = current.is_next(result.did.clone());
        assert!(r.is_ok());
    }

    #[test]
    fn is_next_invaliddoc_test() {
        let mut result = create_did();
        let current = result.did.clone();
        let secret = multibase::decode("beilmx4d76udjmug5ykpy657qa3pfsqbcu7fbbtuk3mgrdrxssseq")
            .unwrap()
            .1;
        result.did.did_doc = IdDocument::new_with_secrets(
            result.did.id.clone(),
            secret.clone(),
            secret.clone(),
            secret.clone(),
        )
        .doc;
        let is_next = current.is_next(result.did.clone());
        assert!(matches!(
            is_next,
            Err(crate::IdentityError::InvalidDocumentDigest)
        ));
    }

    fn create_did() -> CreateIdentityResult {
        let signer_secret =
            multibase::decode("beilmx4d76udjmug5ykpy657qa3pfsqbcu7fbbtuk3mgrdrxssseq")
                .unwrap()
                .1;
        let recovery_secret =
            multibase::decode("blunvrc23gte2nwj7cbf3sjszie7ti3bc6xk257a6rfjcsxwxpuwa")
                .unwrap()
                .1;
        let assertion_secret = create_secret_key();
        let authentication_secret = create_secret_key();
        let keyagreement_secret = create_secret_key();
        Identity::new_with_secrets(
            signer_secret,
            recovery_secret,
            assertion_secret,
            authentication_secret,
            keyagreement_secret,
        )
    }
}
