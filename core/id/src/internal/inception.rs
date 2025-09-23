use super::{claim::IdClaimCreateEvent, signer::IdSigner};
use alloc::collections::BTreeSet;
use alloc::string::String;
use cid::Cid;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub struct IdInception {
    pub version: String,
    pub patch: Cid,
    pub timestamp: i64,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub prior_id: Option<String>,
    pub threshold: u8,
    pub next_threshold: u8,
    pub signers: BTreeSet<IdSigner>,
    pub next_signers: BTreeSet<String>,
    #[serde(skip_serializing_if = "BTreeSet::is_empty", default)]
    pub delegators: BTreeSet<String>,
    #[serde(skip_serializing_if = "BTreeSet::is_empty", default)]
    pub claims: BTreeSet<IdClaimCreateEvent>,
}

