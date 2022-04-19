use idp2p_common::chrono::Utc;
use idp2p_common::serde_json;
use crate::did::identity::Identity;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub enum IdNodeRequestPayload {
    Register,
    Subscribe(String),
    Publish(IdGossipMessage),
    Get(String)
}

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub enum IdWalletRequestPayload {
    
}

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub enum IdResponsePayload {
    Ok,
    Error(String)
}

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub struct IdGossipMessage {
    pub id: String,
    pub timestamp: i64,
    pub payload: IdGossipMessagePayload,
}

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
#[serde(tag = "type")]
pub enum IdGossipMessagePayload {
    #[serde(rename = "get")]
    Get,
    #[serde(rename = "post")]
    Post { digest: String, identity: Identity },
    #[serde(rename = "jwm")]
    Jwm { jwm: String },
}

impl IdGossipMessage {
    pub fn new_post(did: Identity) -> Self {
        let id = did.id.clone();
        let payload = IdGossipMessagePayload::Post {
            digest: did.get_digest(),
            identity: did,
        };
        Self::new(&id, payload)
    }

    pub fn new_get(id: &str) -> Self {
        Self::new(id, IdGossipMessagePayload::Get)
    }

    pub fn new_jwm(id: &str, jwm: &str) -> Self {
        let payload = IdGossipMessagePayload::Jwm {
            jwm: jwm.to_owned(),
        };
        Self::new(id, payload)
    }

    fn new(id: &str, payload: IdGossipMessagePayload) -> Self {
        let message = Self {
            id: id.to_owned(),
            timestamp: Utc::now().timestamp(),
            payload: payload,
        };
        message
    }

    pub fn from_bytes(mes: &[u8]) -> Self {
        let err_msg = "Message is not well-formed. It should be json";
        serde_json::from_slice(mes).expect(err_msg)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn new_test() {
        let message = IdGossipMessage::new_get("adem");
        assert_eq!(message.id, "adem");
    }
}
