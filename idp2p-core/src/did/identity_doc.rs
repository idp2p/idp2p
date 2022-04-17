use idp2p_common::{
     encode_vec, hash, serde_json, serde_with::skip_serializing_none
};
use serde::{Deserialize, Serialize};

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
    #[serde(rename = "@context")]
    pub context: Vec<String>,
    pub id: String,
    pub controller: String,
    #[serde(rename = "verificationMethod")]
    pub verification_method: Vec<VerificationMethod>,
    #[serde(rename = "assertionMethod")]
    pub assertion_method: Vec<String>,
    pub authentication: Vec<String>,
    #[serde(rename = "keyAgreement")]
    pub key_agreement: Vec<String>,
}

impl IdDocument {
    pub fn get_digest(&self) -> Vec<u8> {
        hash(serde_json::to_string(&self).unwrap().as_bytes())
    }

    pub fn get_verification_method(&self, kid: &str) -> Option<VerificationMethod> {
        self.verification_method
            .clone()
            .into_iter()
            .find(|vm| vm.id == kid)
    }

    pub fn get_first_keyagreement(&self) -> String {
        self.key_agreement[0].clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use idp2p_common::ed_secret::EdSecret;
    /*#[test]
    fn new_did_doc_test() {
        let secret = EdSecret::new();
        let ed_key = secret.to_publickey();
        let x_key = secret.to_key_agreement();
        let input = CreateDocInput {
            id: "did:p2p:123456".to_owned(),
            assertion_key: ed_key.to_vec(),
            authentication_key: ed_key.to_vec(),
            keyagreement_key: x_key.to_vec(),
        };
        let doc = IdDocument::new(input);
        assert_eq!(doc.id, "did:p2p:123456");
        assert_eq!(doc.controller, "did:p2p:123456");
        assert_eq!(doc.verification_method.len(), 3);
    }

    #[test]
    fn get_verification_method_test() {
        let secret = EdSecret::new();
        let ed_key = secret.to_publickey();
        let x_key = secret.to_key_agreement();
        let input = CreateDocInput {
            id: "did:p2p:123456".to_owned(),
            assertion_key: ed_key.to_vec(),
            authentication_key: ed_key.to_vec(),
            keyagreement_key: x_key.to_vec(),
        };
        let doc = IdDocument::new(input);
        let kid = format!("did:p2p:123456#{}", encode(&x_key));
        let vm = doc.get_verification_method(&kid);
        assert!(vm.is_some());
    }*/
}
