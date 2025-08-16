use serde::{Deserialize, Serialize};
use serde_with::serde_as;
use idp2p_common::bytes::Bytes;

#[serde_as]
#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub struct IdEventProof {
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
pub struct IdProof {
    // The identity who creates proof
    pub id: String,

    // Proof version
    pub version: String,

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

#[serde_as]
#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub struct IdEventEnvelope {
    pub id: String,
    pub version: String,
    #[serde_as(as = "Bytes")]
    pub payload: Vec<u8>,
    // Key means kid, value means signature
    pub proofs: Vec<IdEventProof>,
    // Key means id, value means signature
    pub delegated_proofs: Vec<IdProof>,
}
