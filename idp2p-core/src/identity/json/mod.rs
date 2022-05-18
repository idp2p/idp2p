use idp2p_common::{
    encode_vec, key::{Idp2pKey,Idp2pAgreementKey}, digest::{Idp2pKeyDigest, Idp2pDigest},
};
use serde::{Deserialize, Serialize};
pub mod handler;
pub mod mapper;
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
    pub version: i32,
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
    #[serde(with = "encode_vec")]
    pub key: Vec<u8>,
    #[serde(with = "encode_vec")]
    pub value: Vec<u8>,
}

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
#[serde(tag = "type")]
pub enum EventType {
    CreateAssertionKey { key: Idp2pKey },
    CreateAuthenticationKey { key: Idp2pKey },
    CreateAgreementKey { key: Idp2pAgreementKey },
    RevokeAssertionKey { kid: Idp2pDigest },
    RevokeAuthenticationKey { kid: Idp2pDigest },
    RevokeAgreementKey { kid: Idp2pDigest },
    SetProof { proof: Idp2pProof },
}

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub struct EventLogPayload {
    pub version: i32,
    pub previous: Idp2pDigest,
    #[serde(with = "encode_vec")]
    pub signer_key: Vec<u8>,
    pub next_key_digest: Idp2pKeyDigest,
    pub timestamp: i64,
    pub change: EventLogChange,
}

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub struct EventLog {
    pub event_id: Idp2pDigest,
    pub payload: EventLogPayload,
    #[serde(with = "encode_vec")]
    pub proof: Vec<u8>, // if recover assume recovery key
}
