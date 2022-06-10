use idp2p_common::{serde_vec::serde_vec, multi::key_digest::Idp2pKeyDigest};
use serde::{Deserialize, Serialize};

use crate::identity::models::{IdEvent, ChangeType};

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub struct Microledger {
    pub inception: IdentityInception,
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub event_logs: Vec<EventLog>,
}

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub struct IdentityInception {
    pub timestamp: i64,
    #[serde(with = "serde_vec")]
    pub next_key_digest: Vec<u8>,
    #[serde(with = "serde_vec")]
    pub recovery_key_digest: Vec<u8>,
    pub events: Vec<IdEvent>,
}

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub struct EventLog {
    #[serde(with = "serde_vec")]
    pub event_id: Vec<u8>,
    pub payload: EventLogPayload,
    #[serde(with = "serde_vec")]
    pub proof: Vec<u8>, // if recover assume recovery key
}

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub struct EventLogPayload {
    #[serde(with = "serde_vec")]
    pub previous: Vec<u8>,
    #[serde(with = "serde_vec")]
    pub signer_key: Vec<u8>,
    #[serde(with = "serde_vec")]
    pub next_key_digest: Vec<u8>,
    pub timestamp: i64,
    pub change: ChangeType,
}