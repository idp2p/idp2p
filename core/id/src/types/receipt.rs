use std::collections::BTreeSet;

use alloc::str::FromStr;
use cid::Cid;
use idp2p_common::{CBOR_CODE, ED_CODE, bytes::Bytes, cid::CidExt, ed25519};
use serde::{Deserialize, Serialize};
use serde_with::serde_as;

use crate::{verifier::signer::IdSigner, verifier::error::IdEventError};

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

    // Proof version
    pub version: String,

    // Proof purpose
    pub purpose: String,

    // The key which signs the data
    pub key_id: String,

    // Proof time
    pub created_at: String,

    // Proof content hash
    #[serde_as(as = "Bytes")]
    pub content_id: Vec<u8>,

    // Bytes of signature
    #[serde_as(as = "Bytes")]
    pub signature: Vec<u8>,
}

#[serde_as]
#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub struct IdEventReceipt {
    pub id: String,
    pub version: String,
    pub created_at: String,
    #[serde_as(as = "Bytes")]
    pub payload: Vec<u8>,
    // Key means kid, value means signature
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub proofs: Vec<IdProof>,
    // Key means id, value means signature
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub external_proofs: Vec<IdProofReceipt>,
}

impl IdEventReceipt {
    pub fn verify_proofs(&self, signers: &BTreeSet<IdSigner>) -> Result<(), IdEventError> {
        for proof in &self.proofs {
            
        }
        for signer in signers {
            let kid = Cid::from_str(&signer.id)?;
            kid.ensure(&signer.public_key, vec![ED_CODE])?;
            let proof = self
                .proofs
                .iter()
                .find(|p| p.key_id == signer.id)
                .ok_or(IdEventError::LackOfMinProofs)?;

            match kid.codec() {
                ED_CODE => ed25519::verify(&signer.public_key, &self.payload, &proof.signature)?,
                _ => {
                    return Err(IdEventError::InvalidSigner(
                        "Unsupported key type".to_string(),
                    ));
                }
            }
        }
        Ok(())
    }
}
