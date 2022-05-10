use idp2p_common::{encode_vec, key_digest::Idp2pKeyDigest};
use serde::{Deserialize, Serialize};

use super::core::IdentityEvent;

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub struct Identity {
    pub id: String,
    pub microledger: Microledger,
}

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub struct Microledger {
    pub inception: IdentityInception,
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub event_logs: Vec<EventLog>,
}

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub struct IdentityInception {
    pub recovery_key_digest: Idp2pKeyDigest,
    pub next_key_digest: Idp2pKeyDigest,
    //pub events: Vec<IdentityEvent>,
}

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
#[serde(tag = "type")]
pub enum EventLogChange {
    Recover { digest: Idp2pKeyDigest },
    //Events { events: Vec<IdentityEvent> },
}

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub struct EventLogPayload {
    pub previous: String,
    #[serde(with = "encode_vec")]
    pub signer_key: Vec<u8>,
    pub next_key_digest: Idp2pKeyDigest,
    pub timestamp: i64,
    pub change: EventLogChange,
}

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub struct EventLog {
    pub payload: EventLogPayload,
    #[serde(with = "encode_vec")]
    pub proof: Vec<u8>, // if recover assume recovery key
}
