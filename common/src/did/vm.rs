use core::str::FromStr;

use alloc::string::{String, ToString};
use regex::Regex;
use serde::{Deserialize, Serialize};

use crate::error::CommonError;

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
#[serde(untagged)]
pub enum PublicKey {
    Jwk {
        #[serde(rename = "publicKeyJwk")]
        public_key_jwk: PublicJwk,
    },
    Multibase {
        #[serde(rename = "publicKeyMultibase")]
        public_key_multibase: String,
    },
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct PublicJwk {
    pub kty: String,
    pub crv: String,
    pub x: String,
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
    Object(VerificationMethod),
}

#[derive(Deserialize, Serialize, Debug)]
pub struct VerificationMethodId {
    id: String,
    key_id: String,
}

impl ToString for VerificationMethodId {
    fn to_string(&self) -> String {
        format!("/did:p2p:/{}#{}", self.id, self.key_id)
    }
}

impl FromStr for VerificationMethodId {
    type Err = CommonError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let re = Regex::new(r"^did:p2p:(?<id>[a-z]+)/(?<key_id>.+)$")
            .map_err(|e| CommonError::InvalidIdentifier(e.to_string()))?;

        let caps = re.captures(s)
            .ok_or(CommonError::InvalidIdentifier(s.to_string()))?;

        Ok(Self { id: caps["id"].to_string(), key_id: caps["key_id"].to_string() })
    }
}
