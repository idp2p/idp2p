use std::{
    collections::hash_map::DefaultHasher,
    hash::{Hash, Hasher},
    time::Duration,
};

use libp2p::gossipsub::{
    Gossipsub, GossipsubConfigBuilder, GossipsubMessage, IdentTopic, MessageAuthenticity,
    MessageId, ValidationMode,
};
use serde::{Deserialize, Serialize};
use idp2p_common::{anyhow::Result, chrono::Utc, serde_json};
use crate::{did::identity::Identity};

pub fn build_gossipsub() -> Gossipsub {
    let message_id_fn = |message: &GossipsubMessage| {
        let mut s = DefaultHasher::new();
        message.data.hash(&mut s);
        MessageId::from(s.finish().to_string())
    };
    let gossipsub_config = GossipsubConfigBuilder::default()
        .heartbeat_interval(Duration::from_secs(10))
        .validation_mode(ValidationMode::Anonymous)
        .message_id_fn(message_id_fn)
        .build()
        .expect("Valid config");
    let gossipsub_result = Gossipsub::new(MessageAuthenticity::Anonymous, gossipsub_config);
    let gossipsub = gossipsub_result.expect("Correct configuration");
    gossipsub
}

pub trait IdGossip {
    fn subscribe_to(&mut self, id: &str) -> Result<()>;
    fn publish_get(&mut self, id: &str) -> Result<()>;
    fn publish_post(&mut self, did: Identity) -> Result<()>;
}

impl IdGossip for Gossipsub {
    fn subscribe_to(&mut self, id: &str) -> Result<()> {
        let topic = IdentTopic::new(id);
        self.subscribe(&topic)?;
        Ok(())
    }

    fn publish_get(&mut self, id: &str) -> Result<()> {
        let topic = IdentTopic::new(id);
        let msg = IdGossipMessage::new_get();
        idp2p_common::log::info!(
            "Published get: {}",
            idp2p_common::serde_json::to_string_pretty(&msg)?
        );
        let data = idp2p_common::serde_json::to_vec(&msg)?;
        self.publish(topic, data)?;
        Ok(())
    }

    fn publish_post(&mut self, did: Identity) -> Result<()> {
        let topic = IdentTopic::new(&did.id);
        let msg = IdGossipMessage::new_post(did);
        idp2p_common::log::info!(
            "Published post: {:?}",
            idp2p_common::serde_json::to_string_pretty(&msg)?
        );
        let data = idp2p_common::serde_json::to_vec(&msg)?;
        self.publish(topic, data)?;
        Ok(())
    }
}

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub struct IdGossipMessage {
    // Unique gossip message id
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
        let payload = IdGossipMessagePayload::Post {
            digest: did.get_digest(),
            identity: did,
        };
        Self::new(payload)
    }

    pub fn new_get() -> Self {
        Self::new(IdGossipMessagePayload::Get)
    }

    pub fn new_jwm(jwm: &str) -> Self {
        let payload = IdGossipMessagePayload::Jwm {
            jwm: jwm.to_owned(),
        };
        Self::new(payload)
    }

    fn new(payload: IdGossipMessagePayload) -> Self {
        let random: [u8;32] = idp2p_common::create_random();
        let message = Self {
            id: idp2p_common::encode(&random),
            timestamp: Utc::now().timestamp(),
            payload: payload,
        };
        message
    }

    pub fn from_bytes(mes: &[u8]) -> Result<Self> {
        Ok(serde_json::from_slice(mes)?)
    }

    pub fn to_bytes(&self) -> Result<Vec<u8>> {
        Ok(serde_json::to_vec(&self)?)
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn new_test() {
        let message = IdGossipMessage::new_get();
        assert!(!message.id.is_empty());
    }
}
