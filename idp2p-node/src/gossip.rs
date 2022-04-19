use std::{
    collections::hash_map::DefaultHasher,
    hash::{Hash, Hasher},
    sync::Arc,
    time::Duration,
};

use async_trait::async_trait;
use idp2p_core::protocol::IdGossipMessage;
use libp2p::gossipsub::{
    Gossipsub, GossipsubConfigBuilder, GossipsubMessage, IdentTopic, MessageAuthenticity,
    MessageId, ValidationMode,
};

use crate::store::NodeStore;
use idp2p_common::anyhow::Result;

pub(crate) fn build_gossipsub() -> Gossipsub {
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

#[async_trait(?Send)]
pub trait IdGossip {
    fn subscribe_to(&mut self, id: &str) -> Result<()>;
    fn publish_get(&mut self, id: &str) -> Result<()>;
    fn publish_post(&mut self, id: &str, store: Arc<NodeStore>) -> Result<()>;
}

#[async_trait(?Send)]
impl IdGossip for Gossipsub {
    fn subscribe_to(&mut self, id: &str) -> Result<()> {
        let topic = IdentTopic::new(id);
        self.subscribe(&topic)?;
        Ok(())
    }

    fn publish_get(&mut self, id: &str) -> Result<()> {
        let topic = IdentTopic::new(id);
        let msg = IdGossipMessage::new_get(id);
        let data = idp2p_common::serde_json::to_vec(&msg)?;
        self.publish(topic, data)?;
        Ok(())
    }

    fn publish_post(&mut self, id: &str, store: Arc<NodeStore>) -> Result<()> {
        let topic = IdentTopic::new(id);
        let did = store.get_did(id);
        if let Some(did) = did {
            let msg = IdGossipMessage::new_post(did);
            let data = idp2p_common::serde_json::to_vec(&msg)?;
            self.publish(topic, data)?;
        }
        Ok(())
    }
}
