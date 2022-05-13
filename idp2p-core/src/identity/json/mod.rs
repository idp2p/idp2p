use idp2p_common::{
    agreement_key::Idp2pAgreementKey, encode_vec, key::Idp2pKey, key_digest::Idp2pKeyDigest,
};
use serde::{Deserialize, Serialize};
pub mod handler;
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
    pub timestamp: i64,
    pub next_key_digest: Idp2pKeyDigest,
    pub recovery_key_digest: Idp2pKeyDigest,
    pub events: Vec<EventType>,
}

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
#[serde(tag = "type")]
pub enum EventLogChange {
    Recover { digest: Idp2pKeyDigest },
    Events { events: Vec<EventType> },
}

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub struct Idp2pProof {
    pub key: Vec<u8>,
    pub value: Vec<u8>,
}

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
#[serde(tag = "type")]
pub enum EventType {
    CreateAssertionKey { key: Idp2pKey },
    CreateAuthenticationKey { key: Idp2pKey },
    CreateAgreementKey { key: Idp2pAgreementKey },
    RevokeAssertionKey { kid: Vec<u8> },
    RevokeAuthenticationKey { kid: Vec<u8> },
    RevokeAgreementKey { kid: Vec<u8> },
    SetProof { proof: Idp2pProof },
}

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub struct EventLogPayload {
    #[serde(with = "encode_vec")]
    pub previous: Vec<u8>,
    #[serde(with = "encode_vec")]
    pub signer_key: Vec<u8>,
    pub next_key_digest: Idp2pKeyDigest,
    pub timestamp: i64,
    pub change: EventLogChange,
}

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub struct EventLog {
    #[serde(with = "encode_vec")]
    pub event_id: Vec<u8>,
    pub payload: EventLogPayload,
    #[serde(with = "encode_vec")]
    pub proof: Vec<u8>, // if recover assume recovery key
}
