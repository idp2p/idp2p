use crate::bytes::Bytes;
use alloc::{string::String, vec::Vec};
use serde::{Deserialize, Serialize};
use serde_with::serde_as;

#[serde_as]
#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DataIntegrityProof {
    #[serde(rename = "type")]
    pub typ: String,
    pub cryptosuite: String,
    pub created: String,
    pub verification_method: String,
    /// id-inception, id-interaction, id-rotation, id-migration, id-revocation, id-delegation
    pub proof_purpose: String,
    #[serde_as(as = "Bytes")]
    pub proof_value: Vec<u8>,
}
