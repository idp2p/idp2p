use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;
use idp2p_common::serde_vec::serde_vec;

#[skip_serializing_none]
#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub struct VerificationMethod {
    pub id: String,
    pub controller: String,
    #[serde(rename = "type")]
    pub typ: String,
    #[serde(with = "serde_vec", rename = "publicKeyMultibase")]
    pub bytes: Vec<u8>,
}

#[skip_serializing_none]
#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub struct IdentityDocument {
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

impl IdentityDocument {
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