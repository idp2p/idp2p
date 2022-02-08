use crate::did_doc::IdDocument;
use crate::microledger::MicroLedger;
use crate::IdentityError;
use anyhow::Result;
use idp2p_common::*;
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
    pub fn new(base_key_digest: &[u8], next_key_digest: &[u8]) -> Self {
        let ledger = MicroLedger::new(base_key_digest, next_key_digest);
        let id = ledger.inception.get_id();
        let did = Identity {
            id: id.clone(),
            microledger: ledger,
            document: None,
        };
        did
    }

    pub fn verify(&self) -> Result<bool, IdentityError> {
        let id = self.microledger.inception.get_id();
        let verified = self.microledger.verify(&id)?;
        if let Some(document) = self.document.clone() {
            let doc_json = serde_json::to_string(&document).unwrap();
            let did_doc_digest = hash(doc_json.as_bytes());
            check!(
                did_doc_digest == verified.document_digest,
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
        let doc_valid = !(self.document.is_some() && new_did.document.is_none());
        check!(doc_valid, IdentityError::InvalidDocumentDigest);
        candidate.document = new_did.document.clone();
        let did_valid = candidate.get_digest() == new_did.get_digest();
        check!(did_valid, IdentityError::InvalidNext);
        candidate.verify()
    }

    pub fn get_digest(&self) -> String {
        let digest = hash(serde_json::to_string(&self).unwrap().as_bytes());
        encode(&digest)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::did_doc::CreateDocInput;
    use crate::eventlog::DocumentDigest;
    use crate::eventlog::EventLogChange;

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
        save_doc(&mut did, secret);
        let r = current.is_next(did.clone());
        assert!(r.is_ok(), "{:?}", r);
    }

    #[test]
    fn is_next_invalid_doc_test_for_empty_doc() {
        let (mut did, secret) = create_did();
        save_doc(&mut did, secret);
        let current = did.clone();
        did.document = None;
        let result = current.is_next(did);
        let is_err = matches!(result, Err(crate::IdentityError::InvalidDocumentDigest));
        assert!(is_err, "{:?}", result);
    }

    #[test]
    fn is_next_invalid_doc_test() {
        let (mut did, secret) = create_did();
        save_doc(&mut did, secret.clone());
        let current = did.clone();
        let digest = DocumentDigest { value: vec![] };
        let fake_change = EventLogChange::SetDocument(digest);
        let fake_payload = did.microledger.create_event(
            &secret.to_verification_publickey(),
            &secret.to_publickey_digest(),
            fake_change,
        );
        let fake_proof = secret.sign(&fake_payload);
        did.microledger.save_event(fake_payload, &fake_proof);
        let result = current.is_next(did);
        let is_err = matches!(result, Err(crate::IdentityError::InvalidDocumentDigest));
        assert!(is_err, "{:?}", result);
    }

    fn create_did() -> (Identity, secret::IdSecret) {
        let secret_str = "beilmx4d76udjmug5ykpy657qa3pfsqbcu7fbbtuk3mgrdrxssseq";
        let secret = secret::IdSecret::from(&decode(secret_str));
        let ed_key_digest = secret.to_publickey_digest();
        (Identity::new(&ed_key_digest, &ed_key_digest), secret)
    }

    fn save_doc(did: &mut Identity, secret: secret::IdSecret) {
        let ed_key = secret.to_verification_publickey();
        let x_key = secret.to_key_agreement_publickey();
        let input = CreateDocInput {
            id: did.id.clone(),
            assertion_key: ed_key.clone(),
            authentication_key: ed_key.clone(),
            keyagreement_key: x_key.clone(),
        };
        let doc = IdDocument::new(input);
        let doc_digest = doc.get_digest();
        did.document = Some(doc);
        let change = EventLogChange::SetDocument(DocumentDigest { value: doc_digest });
        let signer = secret.to_verification_publickey();
        let payload = did
            .microledger
            .create_event(&signer, &hash(&signer), change);
        let proof = secret.sign(&payload);
        did.microledger.save_event(payload, &proof);
    }
}
