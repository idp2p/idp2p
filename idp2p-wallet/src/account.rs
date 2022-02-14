use idp2p_common::encode_vec;
use idp2p_core::ver_cred::VerifiableCredential;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub struct WalletAccount {
    pub name: String,
    pub id: String,
    pub next_derivation_index: u32,
    pub recovery_derivation_index: u32,
    pub assertion_derivation_index: Option<u32>,
    pub authentication_derivation_index: Option<u32>,
    pub keyagreement_derivation_index: Option<u32>,
    pub credentials: Option<Vec<VerifiableCredential>>,
}

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub struct WalletAccountSecret {
    #[serde(with = "encode_vec")]
    pub next_secret: Vec<u8>,
    #[serde(with = "encode_vec")]
    pub recovery_secret: Vec<u8>,
    #[serde(with = "encode_vec")]
    pub assertion_secret: Vec<u8>,
    #[serde(with = "encode_vec")]
    pub authentication_secret: Vec<u8>,
    #[serde(with = "encode_vec")]
    pub keyagreement_secret: Vec<u8>,
}