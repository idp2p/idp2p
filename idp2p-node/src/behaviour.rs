use idp2p_core::protocol::id_message::IdentityMessage;
use crate::id_store::IdNodeStore;
use std::sync::Arc;
use idp2p_common::anyhow::Result;
use async_trait::async_trait;
use libp2p::PeerId;
use libp2p::{
    gossipsub::{Gossipsub, GossipsubEvent, IdentTopic},
    mdns::{Mdns, MdnsEvent},
    NetworkBehaviour,
};

#[derive(NetworkBehaviour)]
#[behaviour(out_event = "IdentityNodeEvent")]
pub struct IdentityNodeBehaviour {
    pub(crate) mdns: Mdns,
    pub(crate) gossipsub: Gossipsub,
    #[behaviour(ignore)]
    pub store: Arc<IdNodeStore>,
}

#[derive(Debug)]
pub enum IdentityNodeEvent {
    Gossipsub(GossipsubEvent),
    Mdns(MdnsEvent)
}

impl From<GossipsubEvent> for IdentityNodeEvent {
    fn from(event: GossipsubEvent) -> Self {
        IdentityNodeEvent::Gossipsub(event)
    }
}

impl From<MdnsEvent> for IdentityNodeEvent {
    fn from(event: MdnsEvent) -> Self {
        IdentityNodeEvent::Mdns(event)
    }
}

impl IdentityNodeBehaviour {
    pub async fn handle_gossip_event(&mut self, event: GossipsubEvent) {
        if let GossipsubEvent::Subscribed { peer_id, topic } = event {
            if self.is_authorized_peer(peer_id) {
                let new_topic = IdentTopic::new(topic.into_string());
                self.gossipsub.subscribe(&new_topic).unwrap();
            }
        }
    }

    fn is_authorized_peer(&self, _peer_id: PeerId) -> bool {
        true
    }
}

#[async_trait(?Send)]
pub trait IdGossip {
    fn subscribe_to(&mut self, id: &str) -> Result<()>;
    fn publish_get(&mut self, id: &str) -> Result<()>;
    fn publish_msg(&mut self, id: &str, store: Arc<IdNodeStore>) -> Result<()>;
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
        let msg = IdentityMessage::new_get(id);
        let data = idp2p_common::serde_json::to_vec(&msg)?;
        self.publish(topic, data)?;
        Ok(())
    }

    fn publish_msg(&mut self, id: &str, store: Arc<IdNodeStore>) -> Result<()> {
        let topic = IdentTopic::new(id);
        let msg = store.get_message(id);
        if let Some(msg) = msg {
            let data = idp2p_common::serde_json::to_vec(&msg)?;
            self.publish(topic, data)?;
        }
        Ok(())
    }
}