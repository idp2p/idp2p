use crate::create_key_agreement;
use crate::create_verification_key;
use crate::did_doc::IdDocument;
use crate::encode_me;
use crate::eventlog::DocumentDigest;
use crate::eventlog::EventLog;
use crate::eventlog::EventLogChange;
use crate::eventlog::EventLogPayload;
use crate::eventlog::ProofStatement;
use crate::eventlog::RecoverStatement;
use crate::hash;
use crate::microledger::MicroLedger;
use crate::IdentityError;
use crate::RecoveryKey;
use crate::SignerKey;
use serde::{Deserialize, Serialize};
use std::convert::TryInto;

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub struct Identity {
    pub ledger: MicroLedger,
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

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub struct RecoveryResult {
    pub did: Identity,
    #[serde(with = "encode_me")]
    pub signer_secret: Vec<u8>,
    #[serde(with = "encode_me")]
    pub recovery_secret: Vec<u8>,
}

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub struct CreateDocResult {
    pub doc: IdDocument,
    #[serde(with = "encode_me")]
    pub assertion_secret: Vec<u8>,
    #[serde(with = "encode_me")]
    pub authentication_secret: Vec<u8>,
    #[serde(with = "encode_me")]
    pub keyagreement_secret: Vec<u8>,
}

impl Identity {
    pub fn create() -> CreateIdentityResult {
        let (signer_secret, signer_public) = create_verification_key();
        let (recovery_secret, recovery_public) = create_verification_key();
        let recovery_public_bytes: [u8; 32] = recovery_public.try_into().unwrap();
        let ledger = MicroLedger::new(signer_public, hash(&recovery_public_bytes));
        let doc_result = Identity::create_doc(ledger.id.clone());
        let mut did = Identity {
            ledger: ledger,
            did_doc: doc_result.doc.clone(),
        };
        did.set_doc_proof(doc_result.doc.clone(), signer_secret.clone());
        CreateIdentityResult {
            did,
            signer_secret,
            recovery_secret,
            assertion_secret: doc_result.assertion_secret,
            authentication_secret: doc_result.authentication_secret,
            keyagreement_secret: doc_result.keyagreement_secret,
        }
    }

    pub fn create_doc(id: String) -> CreateDocResult {
        let (assertion_secret, assertion_public) = create_verification_key();
        let (authentication_secret, authentication_public) = create_verification_key();
        let (keyagreement_secret, keyagreement_public) = create_key_agreement();
        let doc = IdDocument::new(
            id,
            assertion_public,
            authentication_public,
            keyagreement_public,
        );
        CreateDocResult {
            doc: doc,
            assertion_secret: assertion_secret,
            authentication_secret: authentication_secret,
            keyagreement_secret: keyagreement_secret,
        }
    }

    pub fn set_doc_proof(&mut self, doc: IdDocument, secret_key: Vec<u8>) {
        let change = EventLogChange::SetDocument(DocumentDigest {
            value: doc.to_hash(),
        });
        let keypair = crate::to_keypair(secret_key.clone());
        let payload = EventLogPayload {
            previous: self.ledger.get_previous_id(),
            change: change,
            signer_key: keypair.public.as_bytes().to_vec(),
        };
        let event_log = EventLog::create(payload, secret_key.clone());
        self.ledger.events.push(event_log);
    }

    pub fn set_proof(&mut self, secret_key: Vec<u8>, key: Vec<u8>, value: Vec<u8>) {
        let proof_stmt = ProofStatement {
            key: key,
            value: value,
        };
        let change = EventLogChange::SetProof(proof_stmt);
        let keypair = crate::to_keypair(secret_key.clone());
        let payload = EventLogPayload {
            previous: self.ledger.get_previous_id(),
            change: change,
            signer_key: keypair.public.as_bytes().to_vec(),
        };
        let event_log = EventLog::create(payload, secret_key.clone());
        self.ledger.events.push(event_log);
    }

    pub fn recover(&mut self, secret_key: Vec<u8>) -> RecoveryResult {
        let (signer_secret, signer_public) = create_verification_key();
        let (recovery_secret, recovery_public) = create_verification_key();
        let change = EventLogChange::Recover(RecoverStatement {
            next_signer_key: SignerKey::new(signer_public),
            next_recovery_key: RecoveryKey::new(hash(&recovery_public)),
        });
        let keypair = crate::to_keypair(secret_key.clone());
        let payload = EventLogPayload {
            previous: self.ledger.get_previous_id(),
            change: change,
            signer_key: keypair.public.as_bytes().to_vec(),
        };
        let event_log = EventLog::create(payload, secret_key.clone());
        self.ledger.events.push(event_log);
        RecoveryResult {
            did: self.clone(),
            signer_secret: signer_secret,
            recovery_secret: recovery_secret,
        }
    }

    pub fn is_next(&self, new_did: Identity) -> Result<bool, IdentityError> {
        let mut candidate = self.clone();
        let last_id = candidate.ledger.events.last().unwrap().get_id();
        let mut is_last = false;
        for event in &new_did.ledger.events {
            if is_last {
                candidate.ledger.events.push(event.clone());
            }
            if event.get_id() == last_id {
                is_last = true;
            }
        }
        let verified = candidate
            .ledger
            .verify(candidate.ledger.inception.get_id())?;
        let did_valid = candidate.get_digest() == new_did.get_digest();
        check!(did_valid, IdentityError::InvalidLedger);
        let did_doc_digest = hash(serde_json::to_string(&self.did_doc).unwrap().as_bytes());
        check!(
            did_doc_digest == verified.current_doc_digest,
            IdentityError::InvalidLedger
        );
        Ok(true)
    }

    pub fn get_digest(&self) -> String {
        let digest = hash(serde_json::to_string(&self).unwrap().as_bytes());
        crate::encode(digest)
    }
}
