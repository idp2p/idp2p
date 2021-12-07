use crate::encode_me;
use crate::hash;
use crate::ED25519;
use crate::X25519;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub struct Service {
    pub id: String,
    #[serde(rename = "type")]
    pub typ: String,
    #[serde(rename = "serviceEndpoint")]
    pub endpoint: String,
}

#[skip_serializing_none]
#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub struct VerificationMethod {
    pub id: String,         // root did:cid:xxxx#public
    pub controller: String, // root did:cid:xxxx
    #[serde(rename = "type")]
    pub typ: String,
    #[serde(with = "encode_me", rename = "publicKeyMultibase")]
    pub bytes: Vec<u8>,
}

#[skip_serializing_none]
#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub struct IdDocument {
    pub id: String,
    pub controller: String,
    #[serde(rename = "@context")]
    pub context: Vec<String>,
    #[serde(rename = "verificationMethod")]
    pub verification_method: Vec<VerificationMethod>,
    #[serde(rename = "assertionMethod")]
    pub assertion_method: Vec<String>,
    #[serde(rename = "service")]
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub services: Vec<Service>,
    pub authentication: Vec<String>,
    #[serde(rename = "keyAgreement")]
    pub key_agreement: Vec<String>,
}

impl IdDocument {
    pub fn new(
        id: String,
        assertion_public: Vec<u8>,
        authentication_public: Vec<u8>,
        agreement_public: Vec<u8>,
    ) -> IdDocument {
        let assertion_method = VerificationMethod {
            id: crate::encode(assertion_public.clone()),
            controller: format!("did:p2p:{}", id.clone()),
            typ: ED25519.to_string(),
            bytes: assertion_public.clone(),
        };
        let authentication = VerificationMethod {
            id: crate::encode(authentication_public.clone()),
            controller: format!("did:p2p:{}", id.clone()),
            typ: ED25519.to_string(),
            bytes: authentication_public.clone(),
        };
        let key_agreement = VerificationMethod {
            id: crate::encode(agreement_public.clone()),
            controller: format!("did:p2p:{}", id.clone()),
            typ: X25519.to_string(),
            bytes: agreement_public.clone(),
        };
        let doc = IdDocument {
            context: vec![
                "https://www.w3.org/ns/did/v1".to_string(),
                "https://w3id.org/security/suites/ed25519-2020/v1".to_string(),
                "https://w3id.org/security/suites/x25519-2020/v1".to_string(),
            ],
            id: format!("did:p2p:{}", id.clone()),
            controller: format!("did:p2p:{}", id.clone()),
            verification_method: vec![
                assertion_method.clone(),
                authentication.clone(),
                key_agreement.clone(),
            ],
            authentication: vec![authentication.id.clone()],
            assertion_method: vec![assertion_method.id.clone()],
            key_agreement: vec![key_agreement.id.clone()],
            services: vec![],
        };
        doc
    }

    pub fn to_hash(&self) -> Vec<u8> {
        hash(serde_json::to_string(&self).unwrap().as_bytes())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::*;
    #[test]
    fn new_did_doc() {
        let assertion_public = to_verification_publickey(create_secret_key());
        let authentication_public = to_verification_publickey(create_secret_key());
        let agreement_public = to_key_agreement_publickey(create_secret_key());
        let doc = IdDocument::new(
            "123456".to_string(),
            assertion_public,
            authentication_public,
            agreement_public,
        );
        assert_eq!(doc.id, "did:p2p:123456");
        assert_eq!(doc.controller, "did:p2p:123456");
        assert_eq!(doc.verification_method.len(), 3);
    }
}
