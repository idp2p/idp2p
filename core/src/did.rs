use crate::did_doc::CreateDocInput;
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
    pub document: Option<IdDocument>,
}

impl Identity {
    pub fn new(next_key_digest: &[u8], rec_key_digest: &[u8]) -> Identity {
        let ledger = MicroLedger::new(next_key_digest, rec_key_digest);
        let id = ledger.inception.get_id();
        let did = Identity {
            id: id.clone(),
            microledger: ledger,
            document: None,
        };
        did
    }

    pub fn create_document(&mut self, input: CreateDocInput) {
        let signer_secret = &input.signer_secret.to_owned();
        let next_key_digest = &input.next_key_digest.to_owned();
        let document = IdDocument::new(input);
        self.document = Some(document.clone());
        let digest = DocumentDigest {
            value: hash(&document.get_digest()),
        };
        let change = EventLogChange::SetDocument(digest);
        self.save_event(signer_secret, next_key_digest, change)
    }

    pub fn save_event(
        &mut self,
        signer_secret: &[u8],
        next_key_digest: &[u8],
        change: EventLogChange,
    ) {
        let signer_publickey = to_verification_publickey(&signer_secret);
        let previous = self.microledger.get_previous_id();
        let payload = EventLogPayload {
            previous: previous,
            signer_key: signer_publickey,
            next_key_digest: next_key_digest.to_owned(),
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
        crate::encode(&digest)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn set_doc_test() {
        /*let mut did = create_did();
        let old_doc_authentication = did.document.authentication[0].clone();
        let new_doc = IdDocument::new(result.did.id.clone());
        let doc_digest = new_doc.doc.get_digest();
        result.did.document = new_doc.doc;
        let change = EventLogChange::SetDocument(DocumentDigest { value: doc_digest });
        let next_key = hash(&vec![]);
        result.did.save_event(&result.next_secret, next_key, change);
        assert_eq!(result.did.microledger.events.len(), 2);
        assert_ne!(
            result.did.document.authentication[0],
            old_doc_authentication
        );*/
    }

    #[test]
    fn is_next_ok_test() {
        let (mut did, secret) = create_did();
        let ed_key = to_verification_publickey(&secret);
        let x_key = to_key_agreement_publickey(&secret);
        let current = did.clone();
        let input = CreateDocInput{
            id: "123456",
            signer_secret: &secret,
            next_key_digest: &hash(&ed_key),
            recovery_key_digest: &hash(&ed_key),
            assertion_key: &ed_key,
            authentication_key: &ed_key,
            keyagreement_key: &x_key,
            service: &vec![],
        };
        did.create_document(input);
        let r = current.is_next(did.clone());
        assert!(r.is_ok());
    }

    #[test]
    fn is_next_invaliddoc_test() {
        /*let mut result = create_did();
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
        ));*/
    }

    fn create_did() -> (Identity, Vec<u8>) {
        let secret_str = "beilmx4d76udjmug5ykpy657qa3pfsqbcu7fbbtuk3mgrdrxssseq";
        let secret = multibase::decode(secret_str).unwrap().1;
        (Identity::new(&secret, &secret), secret)
    }
}
