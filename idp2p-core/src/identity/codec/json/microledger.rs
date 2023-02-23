use idp2p_common::{serde_vec::serde_vec, multi::key_digest::Idp2pKeyDigest};
use serde::{Serialize, Deserialize};

// Can be used new identity or change
#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
#[serde(tag = "event_type")]
pub enum IdEvent {
    CreateAssertionKey {
        #[serde(with = "serde_vec")]
        id: Vec<u8>,
        #[serde(with = "serde_vec")]
        key: Vec<u8>,
    },
    CreateAuthenticationKey {
        #[serde(with = "serde_vec")]
        id: Vec<u8>,
        #[serde(with = "serde_vec")]
        key: Vec<u8>,
    },
    CreateAgreementKey {
        #[serde(with = "serde_vec")]
        id: Vec<u8>,
        #[serde(with = "serde_vec")]
        key: Vec<u8>,
    },
    SetProof {
        #[serde(with = "serde_vec")]
        key: Vec<u8>,
        #[serde(with = "serde_vec")]
        value: Vec<u8>,
    },
    RevokeAssertionKey(#[serde(with = "serde_vec")] Vec<u8>),
    RevokeAuthenticationKey(#[serde(with = "serde_vec")] Vec<u8>),
    RevokeAgreementKey(#[serde(with = "serde_vec")] Vec<u8>),
}

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
#[serde(tag = "type")]
pub enum ChangeType {
    AddEvents{ events: Vec<IdEvent>},
    Recover(#[serde(with = "serde_vec")] Vec<u8>),
}



#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub struct Microledger {
    #[serde(with = "serde_vec")]
    pub inception: Vec<u8>,
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub event_logs: Vec<EventLog>,
}

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub struct Idp2pInception {
    #[serde(with = "serde_vec")]
    pub next_key_digest: Vec<u8>,
    #[serde(with = "serde_vec")]
    pub recovery_key_digest: Vec<u8>
}

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub struct EventLog {
    #[serde(with = "serde_vec")]
    pub event_id: Vec<u8>,
    #[serde(with = "serde_vec")]
    pub payload: Vec<u8>,
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