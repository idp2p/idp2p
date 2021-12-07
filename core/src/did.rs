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
use crate::to_keypair;
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
        let (signer_secret, _) = create_verification_key();
        let (recovery_secret, _) = create_verification_key();
        Identity::create_with_secrets(signer_secret, recovery_secret)
    }

    pub fn create_with_secrets(
        signer_secret: Vec<u8>,
        rec_secret: Vec<u8>,
    ) -> CreateIdentityResult {
        let rec_public = to_keypair(rec_secret.clone()).public.to_bytes().to_vec();
        let signer_public = to_keypair(signer_secret.clone()).public.to_bytes().to_vec();
        let recovery_public_bytes: [u8; 32] = rec_public.try_into().unwrap();
        let ledger = MicroLedger::new(signer_public, hash(&recovery_public_bytes));
        let doc_result = Identity::create_doc(ledger.id.clone());
        let mut did = Identity {
            ledger: ledger,
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

    pub fn set_doc(&mut self, secret_key: Vec<u8>) -> CreateDocResult {
        let doc_result = Identity::create_doc(self.ledger.id.clone());
        self.did_doc = doc_result.doc.clone();
        self.set_doc_proof(doc_result.doc.clone(), secret_key);
        doc_result
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
            signer_publickey: keypair.public.as_bytes().to_vec(),
        };
        let event_log = EventLog::new(payload, secret_key.clone());
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
            signer_publickey: keypair.public.as_bytes().to_vec(),
        };
        let event_log = EventLog::new(payload, secret_key.clone());
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
        candidate.did_doc = new_did.did_doc.clone();
        let verified = candidate
            .ledger
            .verify(candidate.ledger.inception.get_id())?;
        let did_valid = candidate.get_digest() == new_did.get_digest();
        check!(did_valid, IdentityError::InvalidNext);
        let did_doc_digest = hash(
            serde_json::to_string(&candidate.did_doc)
                .unwrap()
                .as_bytes(),
        );
        check!(
            did_doc_digest == verified.current_doc_digest,
            IdentityError::InvalidDocumentDigest
        );
        Ok(true)
    }

    pub fn get_digest(&self) -> String {
        let digest = hash(serde_json::to_string(&self).unwrap().as_bytes());
        crate::encode(digest)
    }

    fn create_doc(id: String) -> CreateDocResult {
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

    fn set_doc_proof(&mut self, doc: IdDocument, secret_key: Vec<u8>) {
        let change = EventLogChange::SetDocument(DocumentDigest {
            value: doc.to_hash(),
        });
        let keypair = crate::to_keypair(secret_key.clone());
        let payload = EventLogPayload {
            previous: self.ledger.get_previous_id(),
            change: change,
            signer_publickey: keypair.public.as_bytes().to_vec(),
        };
        let event_log = EventLog::new(payload, secret_key.clone());
        self.ledger.events.push(event_log);
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
        assert_eq!(result.did.ledger.id, id);
    }

    #[test]
    fn set_doc() {
        let mut result = create_did();
        let old_doc_authentication = result.did.did_doc.authentication[0].clone();
        result.did.set_doc(result.signer_secret.clone());
        assert_eq!(result.did.ledger.events.len(), 2);
        assert_ne!(result.did.did_doc.authentication[0], old_doc_authentication);
    }

    fn create_did() -> CreateIdentityResult{   
        let signer_secret =
            multibase::decode("beilmx4d76udjmug5ykpy657qa3pfsqbcu7fbbtuk3mgrdrxssseq")
                .unwrap()
                .1;
        let recovery_secret =
            multibase::decode("blunvrc23gte2nwj7cbf3sjszie7ti3bc6xk257a6rfjcsxwxpuwa")
                .unwrap()
                .1;
        Identity::create_with_secrets(signer_secret, recovery_secret)
    }
}
