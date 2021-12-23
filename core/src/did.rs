use crate::did_doc::IdDocument;
use crate::eventlog::DocumentDigest;
use crate::eventlog::EventLog;
use crate::eventlog::EventLogChange;
use crate::eventlog::EventLogPayload;
use crate::microledger::MicroLedger;
use crate::*;
use anyhow::Result;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub struct Identity {
    pub id: String,
    pub microledger: MicroLedger,
    pub document: IdDocument,
}

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub struct CreateIdentityResult {
    pub did: Identity,
    #[serde(with = "encode_me")]
    pub next_secret: Vec<u8>,
    #[serde(with = "encode_me")]
    pub recovery_secret: Vec<u8>,
    #[serde(with = "encode_me")]
    pub assertion_secret: Vec<u8>,
    #[serde(with = "encode_me")]
    pub authentication_secret: Vec<u8>,
    #[serde(with = "encode_me")]
    pub keyagreement_secret: Vec<u8>,
}

impl Identity {
    pub fn new() -> CreateIdentityResult {
        let inception_secret = create_secret_key();
        let next_secret = create_secret_key();
        let recovery_secret = create_secret_key();
        let assertion_secret = create_secret_key();
        let authentication_secret = create_secret_key();
        let key_agreement_secret = create_secret_key();
        Identity::new_with_secrets(
            &inception_secret,
            &next_secret,
            &recovery_secret,
            &assertion_secret,
            &authentication_secret,
            &key_agreement_secret,
        )
    }

    pub fn new_with_secrets(
        inception_secret: &[u8],
        next_secret: &[u8],
        rec_secret: &[u8],
        assertion_secret: &[u8],
        authentication_secret: &[u8],
        keyagreement_secret: &[u8],
    ) -> CreateIdentityResult {
        let rec_public = to_verification_publickey(&rec_secret);
        let inception_public = to_verification_publickey(&inception_secret);
        let next_public = to_verification_publickey(&next_secret);
        let ledger = MicroLedger::new(&inception_public, &rec_public);
        let id = ledger.inception.get_id();
        let doc_result = IdDocument::new_with_secrets(
            id.clone(),
            assertion_secret.to_vec(),
            authentication_secret.to_vec(),
            keyagreement_secret.to_vec(),
        );
        let mut did = Identity {
            id: id.clone(),
            microledger: ledger,
            document: doc_result.doc.clone(),
        };
        let doc_change = EventLogChange::SetDocument(DocumentDigest {
            value: doc_result.doc.get_digest(),
        });
        did.save_event(
            &inception_secret,
            NextKey::from_public(&next_public),
            doc_change,
        );
        CreateIdentityResult {
            did: did,
            next_secret: next_secret.to_vec(),
            recovery_secret: rec_secret.to_vec(),
            assertion_secret: doc_result.assertion_secret,
            authentication_secret: doc_result.authentication_secret,
            keyagreement_secret: doc_result.keyagreement_secret,
        }
    }

    pub fn save_event(&mut self, signer_secret: &[u8], next_key: NextKey, change: EventLogChange) {
        let signer_publickey = to_verification_publickey(&signer_secret);
        let previous = self.microledger.get_previous_id();
        let payload = EventLogPayload {
            previous: previous,
            signer_public_key: signer_publickey,
            next_key: next_key,
            change: change,
        };
        let proof = payload.sign(&signer_secret);
        let event_log = EventLog::new(payload, proof);
        self.microledger.events.push(event_log);
    }

    pub fn verify(&self) -> Result<bool, IdentityError> {
        let id = self.microledger.inception.get_id();
        let verified = self.microledger.verify(&id)?;
        let doc_json = serde_json::to_string(&self.document).unwrap();
        let did_doc_digest = hash(doc_json.as_bytes());
        check!(
            did_doc_digest == verified.doc_digest,
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
        candidate.document = new_did.document.clone();
        let did_valid = candidate.get_digest() == new_did.get_digest();
        check!(did_valid, IdentityError::InvalidNext);
        candidate.verify()
    }

    pub fn get_digest(&self) -> String {
        let digest = hash(serde_json::to_string(&self).unwrap().as_bytes());
        crate::encode(digest)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::eventlog::DocumentDigest;
    #[test]
    fn create_with_secrets_test() {
        let result = create_did();
        let id = "bagaaierap44kwfwbfgfxwcdfjefcqmycu6ongzl3epqyuutg5t6sqeaq53cq";
        assert_eq!(result.did.id, id);
    }

    #[test]
    fn set_doc_test() {
        let mut result = create_did();
        let old_doc_authentication = result.did.document.authentication[0].clone();
        let new_doc = IdDocument::new(result.did.id.clone());
        let doc_digest = new_doc.doc.get_digest();
        result.did.document = new_doc.doc;
        let change = EventLogChange::SetDocument(DocumentDigest { value: doc_digest });
        let next_key = NextKey::from_public(&vec![]);
        result.did.save_event(&result.next_secret, next_key, change);
        assert_eq!(result.did.microledger.events.len(), 2);
        assert_ne!(
            result.did.document.authentication[0],
            old_doc_authentication
        );
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
        result.did.document = IdDocument::new_with_secrets(
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
        let inception_secret =
        multibase::decode("beilmx4d76udjmug5ykpy657qa3pfsqbcu7fbbtuk3mgrdrxssseq")
            .unwrap()
            .1;
        let next_secret =
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
            &inception_secret,
            &next_secret,
            &recovery_secret,
            &assertion_secret,
            &authentication_secret,
            &keyagreement_secret,
        )
    }
}
