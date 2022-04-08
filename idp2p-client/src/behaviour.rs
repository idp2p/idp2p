use async_trait::async_trait;
use idp2p_common::anyhow::Result;
use idp2p_core::{message::IdentityMessage, store::IdStore};
use libp2p::{
    gossipsub::{Gossipsub, GossipsubEvent, IdentTopic},
    mdns::{Mdns, MdnsEvent},
    NetworkBehaviour,
};
use std::sync::Arc;

#[async_trait(?Send)]
pub trait IdGossip {
    fn subscribe_to(&mut self, id: &str) -> Result<()>;
    fn publish_get(&mut self, id: &str) -> Result<()>;
    fn publish_msg(&mut self, id: &str, store: Arc<IdStore>) -> Result<()>;
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

    fn publish_msg(&mut self, id: &str, store: Arc<IdStore>) -> Result<()> {
        let topic = IdentTopic::new(id);
        let msg = store.get_message(id);
        if let Some(msg) = msg {
            let data = idp2p_common::serde_json::to_vec(&msg)?;
            self.publish(topic, data)?;
        }
        Ok(())
    }
}

#[derive(Debug)]
pub enum IdentityClientEvent {
    Mdns(MdnsEvent),
    Gossipsub(GossipsubEvent),
}

impl From<MdnsEvent> for IdentityClientEvent {
    fn from(event: MdnsEvent) -> Self {
        IdentityClientEvent::Mdns(event)
    }
}

impl From<GossipsubEvent> for IdentityClientEvent {
    fn from(event: GossipsubEvent) -> Self {
        IdentityClientEvent::Gossipsub(event)
    }
}

#[derive(NetworkBehaviour)]
#[behaviour(out_event = "IdentityClientEvent")]
pub struct IdentityClientBehaviour {
    pub mdns: Mdns,
    pub gossipsub: Gossipsub,
    #[behaviour(ignore)]
    pub id_store: Arc<IdStore>,
}

impl IdentityClientBehaviour {
    pub fn handle_mdns_event(&mut self, event: MdnsEvent) {
        match event {
            MdnsEvent::Discovered(list) => {
                for (peer, _) in list {
                    self.gossipsub.add_explicit_peer(&peer);
                }
            }
            MdnsEvent::Expired(list) => {
                for (peer, _) in list {
                    if !self.mdns.has_node(&peer) {
                        self.gossipsub.remove_explicit_peer(&peer);
                    }
                }
            }
        }
    }
}
