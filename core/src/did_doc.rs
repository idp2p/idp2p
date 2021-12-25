use crate::*;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

pub struct CreateDocInput {
    pub id: String,
    pub assertion_key: Vec<u8>,
    pub authentication_key: Vec<u8>,
    pub keyagreement_key: Vec<u8>,
    pub service: Vec<Service>,
}

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
    pub id: String,
    pub controller: String,
    #[serde(rename = "type")]
    pub typ: String,
    #[serde(with = "encode_vec", rename = "publicKeyMultibase")]
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
    pub fn new(input: CreateDocInput) -> IdDocument {
        let doc_id = format!("did:p2p:{}", input.id);
        let assertion_id = format!("{}#{}", doc_id, encode(&input.assertion_key));
        let authentication_id = format!("{}#{}", doc_id, encode(&input.authentication_key));
        let keyagreement_id = format!("{}#{}", doc_id, encode(&input.keyagreement_key));
        let assertion_method = VerificationMethod {
            id: assertion_id.clone(),
            controller: doc_id.clone(),
            typ: ED25519.to_string(),
            bytes: input.assertion_key.to_owned(),
        };
        let authentication = VerificationMethod {
            id: authentication_id.clone(),
            controller: doc_id.clone(),
            typ: ED25519.to_string(),
            bytes: input.authentication_key.to_owned(),
        };
        let key_agreement = VerificationMethod {
            id: keyagreement_id.clone(),
            controller: doc_id.clone(),
            typ: X25519.to_string(),
            bytes: input.keyagreement_key.to_owned(),
        };
        let document = IdDocument {
            context: vec![
                "https://www.w3.org/ns/did/v1".to_string(),
                "https://w3id.org/security/suites/ed25519-2020/v1".to_string(),
                "https://w3id.org/security/suites/x25519-2020/v1".to_string(),
            ],
            id: doc_id.clone(),
            controller: doc_id.clone(),
            verification_method: vec![assertion_method, authentication, key_agreement],
            authentication: vec![authentication_id],
            assertion_method: vec![assertion_id],
            key_agreement: vec![keyagreement_id],
            services: vec![],
        };
        document
    }

    pub fn get_digest(&self) -> Vec<u8> {
        hash(serde_json::to_string(&self).unwrap().as_bytes())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn new_did_doc() {
        let secret = create_secret_key();
        let ed_key = to_verification_publickey(&secret);
        let x_key = to_key_agreement_publickey(&secret);
        let input = CreateDocInput{
            id: "123456".to_owned(),
            assertion_key: ed_key.clone(),
            authentication_key: ed_key.clone(),
            keyagreement_key: x_key.clone(),
            service: vec![],
        };
        let doc = IdDocument::new(input);
        assert_eq!(doc.id, "did:p2p:123456");
        assert_eq!(doc.controller, "did:p2p:123456");
        assert_eq!(doc.verification_method.len(), 3);
    }
}
