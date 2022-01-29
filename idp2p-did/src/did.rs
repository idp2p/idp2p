use crate::did_doc::IdDocument;
use crate::eventlog::DocumentDigest;
use crate::eventlog::EventLogChange;
use crate::microledger::MicroLedger;
use crate::*;
use anyhow::Result;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

#[skip_serializing_none]
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

    pub fn create_document(&mut self, next: &[u8], document: IdDocument) -> EventLogChange {
        let digest = DocumentDigest {
            value: document.get_digest(),
        };
        self.document = Some(document);
        let change = EventLogChange::SetDocument(digest);
        
    }

    pub fn verify(&self) -> Result<bool, IdentityError> {
        let id = self.microledger.inception.get_id();
        let verified = self.microledger.verify(&id)?;
        if let Some(document) = self.document.clone() {
            let doc_json = serde_json::to_string(&document).unwrap();
            let did_doc_digest = hash(doc_json.as_bytes());
            check!(
                did_doc_digest == verified.doc_digest,
                IdentityError::InvalidDocumentDigest
            );
        }
        Ok(true)
    }

    pub fn is_next(&self, new_did: Identity) -> Result<bool, IdentityError> {
        let mut candidate = self.clone();
        for event in &new_did.microledger.events {
            if !candidate.microledger.events.contains(&event) {
                candidate.microledger.events.push(event.clone());
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
    use crate::did_doc::CreateDocInput;

    #[test]
    fn is_next_ok_test() {
        let (did, _) = create_did();
        let current = did.clone();
        let r = current.is_next(did.clone());
        assert!(r.is_ok(), "{:?}", r);
    }

    #[test]
    fn is_next_ok_with_doc_test() {
        let (mut did, secret) = create_did();
        let current = did.clone();
        let ed_key = to_verification_publickey(&secret);
        let x_key = to_key_agreement_publickey(&secret);
        let input = CreateDocInput {
            id: did.id.clone(),
            assertion_key: ed_key.clone(),
            authentication_key: ed_key.clone(),
            keyagreement_key: x_key.clone(),
            service: vec![],
        };
        let doc = IdDocument::new(input);
        did.create_document(&secret, &hash(&ed_key), doc);
        let r = current.is_next(did.clone());
        assert!(r.is_ok(), "{:?}", r);
    }

    #[test]
    fn is_next_invalid_doc_test() {
        let (mut did, secret) = create_did();
        let current = did.clone();
        let ed_key = to_verification_publickey(&secret);
        let x_key = to_key_agreement_publickey(&secret);
        let input = CreateDocInput {
            id: did.id.clone(),
            assertion_key: ed_key.clone(),
            authentication_key: ed_key.clone(),
            keyagreement_key: x_key.clone(),
            service: vec![],
        };
        let doc = IdDocument::new(input);
        did.create_document(&secret, &hash(&ed_key), doc);
        let digest = DocumentDigest { value: vec![] };
        let change = EventLogChange::SetDocument(digest);
        did.microledger.save_event(&secret, &secret, change);
        let result = current.is_next(did);
        let is_err = matches!(result, Err(crate::IdentityError::InvalidDocumentDigest));
        assert!(is_err, "{:?}", result);
    }

    fn create_did() -> (Identity, Vec<u8>) {
        let secret_str = "beilmx4d76udjmug5ykpy657qa3pfsqbcu7fbbtuk3mgrdrxssseq";
        let secret = multibase::decode(secret_str).unwrap().1;
        let ed_key_digest = hash(&to_verification_publickey(&secret));
        (Identity::new(&ed_key_digest, &ed_key_digest), secret)
    }
}
