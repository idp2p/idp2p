use idp2p_common::encode_vec;
use idp2p_core::ver_cred::VerifiableCredential;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub struct WalletAccount {
    pub name: String,
    pub id: String,
    #[serde(with = "encode_vec")]
    pub next_secret: Vec<u8>,
    #[serde(with = "encode_vec")]
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub assertion_secret: Vec<u8>,
    #[serde(with = "encode_vec")]
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub authentication_secret: Vec<u8>,
    #[serde(with = "encode_vec")]
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub keyagreement_secret: Vec<u8>,
    pub credentials: Option<Vec<VerifiableCredential>>,
}