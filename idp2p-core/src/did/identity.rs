use super::identity_doc::VerificationMethod;
use super::eventlog::{EventLogChange, EventLogChangeSet};
use super::{identity_doc::IdDocument, microledger::MicroLedger};
use crate::IdentityError;
use idp2p_common::ed_secret::EdSecret;
use idp2p_common::{anyhow::Result, encode, hash, serde_json, serde_with::skip_serializing_none};
use idp2p_common::{log, ED25519, X25519};
use serde::{Deserialize, Serialize};

#[skip_serializing_none]
#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub struct Identity {
    pub id: String,
    pub microledger: MicroLedger,
}

impl Identity {
    pub fn new(recovery_key_digest: &[u8], next_key_digest: &[u8]) -> Self {
        let ledger = MicroLedger::new(recovery_key_digest, next_key_digest);
        let id = ledger.inception.get_id();
        let did = Identity {
            id: format!("did:p2p:{id}"),
            microledger: ledger,
        };
        did
    }

    pub fn from_secret(secret: EdSecret) -> Self {
        let mut did = Identity::new(
            &secret.to_publickey_digest().unwrap(),
            &secret.to_publickey_digest().unwrap(),
        );
        let set_assertion = EventLogChangeSet::SetAssertionKey(VerificationMethod {
            id: format!("{}#{}", did.id.clone(), encode(&secret.to_publickey())),
            controller: did.id.clone(),
            typ: ED25519.to_string(),
            bytes: secret.to_publickey().to_vec(),
        });
        let set_authentication = EventLogChangeSet::SetAuthenticationKey(VerificationMethod {
            id: format!("{}#{}", did.id.clone(), encode(&secret.to_publickey())),
            controller: did.id.clone(),
            typ: ED25519.to_string(),
            bytes: secret.to_publickey().to_vec(),
        });
        let set_agreement = EventLogChangeSet::SetAgreementKey(VerificationMethod {
            id: format!("{}#{}", did.id.clone(), encode(&secret.to_key_agreement())),
            controller: did.id.clone(),
            typ: X25519.to_string(),
            bytes: secret.to_key_agreement().to_vec(),
        });
        let change = EventLogChange::Set {
            sets: vec![set_assertion.clone(), set_authentication, set_agreement],
        };
        let signer = secret.to_publickey();
        let payload = did.microledger.create_event(&signer, &signer, change);
        let proof = secret.sign(&payload);
        did.microledger.save_event(payload, &proof);
        log::info!("Created id: {}", did.id);
        did
    }

    pub fn verify(&self) -> Result<bool, IdentityError> {
        let id = self.microledger.inception.get_id();
        self.microledger.verify(&id)?;
        Ok(true)
    }

    pub fn is_next(&self, new_did: Identity) -> Result<bool, IdentityError> {
        let mut candidate = self.clone();
        for event in &new_did.microledger.events {
            if !candidate.microledger.events.contains(&event) {
                candidate.microledger.events.push(event.clone());
            }
        }
        let did_valid = candidate.get_digest() == new_did.get_digest();
        check!(did_valid, IdentityError::InvalidNext);
        candidate.verify()
    }

    pub fn get_digest(&self) -> String {
        let digest = hash(serde_json::to_string(&self).unwrap().as_bytes());
        encode(&digest)
    }

    pub fn to_document(&self) -> IdDocument {
        let state = self.microledger.verify(self.microledger.inception.get_id().as_str()).unwrap();
        let mut document = IdDocument {
            context: vec![
                "https://www.w3.org/ns/did/v1".to_string(),
                "https://w3id.org/security/suites/ed25519-2020/v1".to_string(),
                "https://w3id.org/security/suites/x25519-2020/v1".to_string(),
            ],
            id: self.id.clone(),
            controller: self.id.clone(),
            verification_method: vec![],
            authentication: vec![],
            assertion_method: vec![],
            key_agreement: vec![],
        };
        if let Some(authentication) = state.authentication_key {
            document.authentication.push(authentication.id.clone());
            document.verification_method.push(authentication);
        }
        if let Some(agreement) = state.agreement_key {
            document.key_agreement.push(agreement.id.clone());
            document.verification_method.push(agreement);
        }
        for assertion in state.assertion_keys {
            document
                .assertion_method
                .push(assertion.ver_method.id.clone());
            document.verification_method.push(assertion.ver_method);
        }
        document
    }
}

#[cfg(test)]
mod tests {
    use crate::did::eventlog::ProofStatement;

    use super::*;
    #[test]
    fn is_next_ok_test() {
        let secret_str = "beilmx4d76udjmug5ykpy657qa3pfsqbcu7fbbtuk3mgrdrxssseq";
        let secret = EdSecret::from_str(secret_str).unwrap();
        let ed_key_digest = secret.to_publickey_digest().unwrap();
        let mut did = Identity::new(&ed_key_digest, &ed_key_digest);
        let previous = did.clone();

        let set_proof = EventLogChangeSet::SetProof(ProofStatement {
            key: vec![1],
            value: vec![1],
        });
        let change = EventLogChange::Set {
            sets: vec![set_proof],
        };
        let signer = secret.to_publickey();
        let payload = did.microledger.create_event(&signer, &signer, change);
        let proof = secret.sign(&payload);
        did.microledger.save_event(payload, &proof);
        let r = previous.is_next(did.clone());
        assert!(r.is_ok(), "{:?}", r);
    }
}
