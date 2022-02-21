use idp2p_core::did_doc::IdDocument;
use idp2p_core::did::Identity;
use idp2p_common::encode_vec;
use idp2p_common::serde_with::skip_serializing_none;
use idp2p_core::ver_cred::VerifiableCredential;
use serde::{Deserialize, Serialize};

#[skip_serializing_none]
#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub struct WalletAccount {
    pub name: String,
    pub did: Identity,
    pub recovery_secret_index: u32,
    pub next_secret_index: u32,
    pub credentials: Option<Vec<VerifiableCredential>>, 
    pub documents: Option<Vec<WalletAccountDocument>>, 
}

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub struct WalletAccountDocument{
    #[serde(with = "encode_vec")]
    pub assertion_secret: Vec<u8>,
    #[serde(with = "encode_vec")]
    pub authentication_secret: Vec<u8>,
    #[serde(with = "encode_vec")]
    pub keyagreement_secret: Vec<u8>,
    pub document: IdDocument,
}