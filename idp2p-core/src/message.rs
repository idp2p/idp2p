use crate::did::Identity;
use idp2p_common::anyhow::Result;
use idp2p_common::chrono::Utc;
use idp2p_common::serde_json;
use libp2p::gossipsub::{Gossipsub, IdentTopic};
use serde::{Deserialize, Serialize};

pub trait Publisher {
    fn publish_msg(&mut self, msg: IdentityMessage) -> Result<()>;
}

impl Publisher for Gossipsub {
    fn publish_msg(&mut self, msg: IdentityMessage) -> Result<()> {
        let topic = IdentTopic::new(&msg.id);
        let data = idp2p_common::serde_json::to_vec(&msg)?;
        self.publish(topic, data)?;
        Ok(())
    }
}

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub struct IdentityMessage {
    pub id: String,
    pub timestamp: i64,
    pub payload: IdentityMessagePayload,
}

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
#[serde(tag = "type")]
pub enum IdentityMessagePayload {
    #[serde(rename = "get")]
    Get,
    #[serde(rename = "post")]
    Post { digest: String, identity: Identity },
    #[serde(rename = "jwm")]
    Jwm { jwm: String },
}

impl IdentityMessage {
    pub fn new_post(did: Identity) -> Self {
        let id = did.id.clone();
        let payload = IdentityMessagePayload::Post {
            digest: did.get_digest(),
            identity: did,
        };
        Self::new(&id, payload)
    }

    pub fn new_get(id: &str) -> Self {
        Self::new(id, IdentityMessagePayload::Get)
    }

    pub fn new_jwm(id: &str, jwm: &str) -> Self {
        let payload = IdentityMessagePayload::Jwm {
            jwm: jwm.to_owned(),
        };
        Self::new(id, payload)
    }

    fn new(id: &str, payload: IdentityMessagePayload) -> Self {
        let message = IdentityMessage {
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
        let message = IdentityMessage::new_get("adem");
        assert_eq!(message.id, "adem");
    }
}
