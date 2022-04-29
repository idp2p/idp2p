use super::{Idp2pPublicKeyDigest, Idp2pRecoveryType, MicroledgerEvent};
use idp2p_common::encode_vec;
use serde::{Deserialize, Serialize};

#[derive(PartialEq, Debug, Clone, Deserialize, Serialize)]
#[serde(tag = "type")]
pub enum EventLogPayloadChange {
    Recover {
        recovery_type: Idp2pRecoveryType,
        recovery_key_digest: Idp2pPublicKeyDigest,
        next_key_digest: Idp2pPublicKeyDigest,
    },
    Events {
        events: Vec<MicroledgerEvent>,
    },
}

#[derive(PartialEq, Debug, Clone, Deserialize, Serialize)]
pub struct EventLog {
    pub payload: EventLogPayload,
    #[serde(with = "encode_vec")]
    pub proof: Vec<u8>,
}

#[derive(PartialEq, Debug, Clone, Deserialize, Serialize)]
pub struct EventLogPayload {
    #[serde(with = "encode_vec")]
    pub previous: Vec<u8>,
    #[serde(with = "encode_vec")]
    pub signer_key: Vec<u8>,
    pub next_key_digest: Idp2pPublicKeyDigest,
    pub timestamp: i64,
    pub change: EventLogPayloadChange,
}
