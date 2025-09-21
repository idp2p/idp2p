
use idp2p_common::bytes::Bytes;
use serde::{Deserialize, Serialize};
use serde_with::serde_as;

#[serde_as]
#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub struct IdProof {
    // The key which signs the data
    pub key_id: String,

    // Proof time
    pub created_at: String,

    // Bytes of signature
    #[serde_as(as = "Bytes")]
    pub signature: Vec<u8>,
}

#[serde_as]
#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub struct IdProofReceipt {
    // The identity who creates proof
    pub id: String,

    // Proof purpose
    pub purpose: String,

    // The key which signs the data
    pub key_id: String,

    // Proof time
    pub created_at: String,

    // Bytes of signature
    #[serde_as(as = "Bytes")]
    pub signature: Vec<u8>,
}