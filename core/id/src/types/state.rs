use idp2p_common::bytes::Bytes;
use serde::{Deserialize, Serialize};
use serde_with::serde_as;

#[serde_as]
#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub struct IdSigner {
    pub id: String,
    /// Public key of the signer.
    #[serde_as(as = "Bytes")]
    pub public_key: Vec<u8>,
    /// Created at sn.
    pub valid_from_sn: u64,
    /// Revoked sn.
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub valid_until_sn: Option<u64>,
    /// Valid from timestamp.
    pub valid_from: String,
    /// Valid to timestamp.
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub valid_until: Option<String>,
}

#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub struct IdState {
    /// Identifier
    pub id: String,

    /// Last event id
    pub event_id: String,

    /// Last event time
    pub event_timestamp: String,

    /// Previous id
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub prior_id: Option<String>,

    /// Next id cid
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub next_id_proof: Option<String>,

    // Current threshold
    pub threshold: u8,

    // Next threshold
    pub next_threshold: u8,

    /// Signers
    pub signers: Vec<IdSigner>,

    /// Current signers
    pub current_signers: Vec<String>,

    /// CID codec should be 0xed
    pub next_signers: Vec<String>,

    /// Delegators
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub delegated_signers: Vec<String>,

    pub merkle_proof: String,

    pub revoked: bool,

    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub revoked_at: Option<String>,
}

