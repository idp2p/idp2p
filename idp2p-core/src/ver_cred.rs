use serde::{Deserialize, Serialize};


#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub struct VerifiableCredential {
    #[serde(rename = "@context")]
    pub context: Vec<String>,
    pub id: String,
    #[serde(rename = "type")]
    pub typ: Vec<String>,
    pub issuer: String,
    #[serde(rename = "issuanceDate")]
    pub issuence_date: String,
    #[serde(rename = "credentialSubject")]
    pub credential_subject: String,
    pub proof: CredentialProof,
}

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub struct CredentialProof {}
