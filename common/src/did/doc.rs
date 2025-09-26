use serde::{Deserialize, Serialize};
use alloc::{string::String, vec::Vec, string::ToString};

use super::vm::{VerificationMethod, VerificationMethodItem};

#[derive(Deserialize, Serialize, Debug)]
#[serde(untagged)]
pub enum StringOrArray {
    String(String),
    Array(Vec<String>),
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct IdDoc {
    #[serde(rename = "@context")]
    pub context: StringOrArray,
    pub id: String,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub controller: Option<StringOrArray>,
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub verification_method: Vec<VerificationMethod>,
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub authentication: Vec<VerificationMethodItem>,
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub assertion_method: Vec<VerificationMethodItem>,
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub key_agreement: Vec<VerificationMethodItem>,
}

impl IdDoc {
    pub fn new(id: &str) -> Self {
        Self {
            context: "https://www.w3.org/ns/did/v1.1".into(),
            id: id.to_string(),
            controller: Some(id.into()),
            verification_method: Vec::new(),
            authentication: Vec::new(),
            assertion_method: Vec::new(),
            key_agreement: Vec::new(),
        }
    }
}

impl From<&str> for StringOrArray {
    fn from(value: &str) -> Self {
        StringOrArray::String(value.to_string())
    }
}

impl From<Vec<&str>> for StringOrArray {
    fn from(value: Vec<&str>) -> Self {
        StringOrArray::Array(value.iter().map(|s| s.to_string()).collect())
    }
}

#[cfg(test)]
mod tests {
    use crate::did::vm::{PublicJwk, PublicKey};

    use super::*;

    #[test]
    fn doc_test() {
        let vm = VerificationMethod {
            id: "did:example:123#key-1".into(),
            r#type: "JsonWebKey2020".into(),
            controller: "did:example:123".into(),
            public_key: PublicKey::Jwk {
                public_key_jwk: PublicJwk {
                    kty: "EC".into(),
                    crv: "P-256".into(),
                    x: "hN6Oq9P8UCj3xqjWYr1Kjh4p2M7YV1o8xjxZrQ4oZ1M".into(),
                },
            },
        };
        let doc = IdDoc {
            context: "https://www.w3.org/ns/did/v1".into(),
            id: "did:example:123".into(),
            controller: None,
            verification_method: vec![vm],
            authentication: vec![],
            assertion_method: vec![],
            key_agreement: vec![],
        };
        let json = serde_json::to_string_pretty(&doc).unwrap();
        println!("{}", json);
    }
}
