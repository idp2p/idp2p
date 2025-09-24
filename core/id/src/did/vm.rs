use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
#[serde(untagged)]
pub enum PublicKey {
   Jwk {
     #[serde(rename = "publicKeyJwk")]
     public_key_jwk: PublicJwk
   },
   Multibase {
     #[serde(rename = "publicKeyMultibase")]
     public_key_multibase: String
   },
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct PublicJwk {
    pub kty: String,
    pub crv: String,
    pub x: String
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct VerificationMethod {
    pub id: String,
    pub r#type: String,
    pub controller: String,
    #[serde(flatten)]
    pub public_key: PublicKey,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(untagged)]
#[serde(rename_all = "camelCase")]
pub enum VerificationMethodItem {
   String(String),
   Object(VerificationMethod)  
}
