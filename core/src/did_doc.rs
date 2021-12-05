use crate::encode_me;
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
    pub id: String,         // / or did:ipfs:xxxxxxx
    pub controller: String, // / or did:ipfs:xxxxxxx
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
            id: multibase::encode(multibase::Base::Base32Lower, assertion_public.clone()),
            controller: format!("did:p2p:{}", id.clone()),
            typ: ED25519.to_string(),
            bytes: assertion_public.clone(),
        };
        let authentication = VerificationMethod {
            id: multibase::encode(multibase::Base::Base32Lower, authentication_public.clone()),
            controller: format!("did:p2p:{}", id.clone()),
            typ: ED25519.to_string(),
            bytes: authentication_public.clone(),
        };
        let key_agreement = VerificationMethod {
            id: multibase::encode(multibase::Base::Base32Lower, agreement_public.clone()),
            controller: format!("did:p2p:{}", id.clone()),
            typ: X25519.to_string(),
            bytes: agreement_public.clone(),
        };
        let doc = IdDocument {
            id: id.clone(),
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
        //println!("{}", serde_json::to_string_pretty(&doc).unwrap());
        doc
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::create_key_agreement;
    use crate::create_verification_key;
    #[test]
    fn generate_test() {
        let (_, assertion_public) = create_verification_key();
        let (_, authentication_public) = create_verification_key();
        let (_, agreement_public) = create_key_agreement();
        let _ = IdDocument::new(
            "123456".to_string(),
            assertion_public,
            authentication_public,
            agreement_public,
        );
    }
}
