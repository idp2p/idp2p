use async_trait::async_trait;
use idp2p_common::anyhow::Result;
use idp2p_core::{
    message::{IdentityMessage, IdentityMessagePayload},
    store::IdStore,
};
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
    async fn handle_event(&mut self, event: GossipsubEvent, id_store: Arc<IdStore>);
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

    async fn handle_event(&mut self, event: GossipsubEvent, id_store: Arc<IdStore>) {
        if let GossipsubEvent::Message {
            propagation_source: _,
            message_id: _,
            message,
        } = event
        {
            let topic = message.topic.to_string();
            if idp2p_common::is_idp2p(&topic) {
                let message = IdentityMessage::from_bytes(&message.data);
                match &message.payload {
                    IdentityMessagePayload::Get => {
                        id_store.handle_get(&topic).await;
                    }
                    IdentityMessagePayload::Post { digest, identity } => {
                        id_store.handle_post(digest, identity).await.unwrap();
                    }
                    _ => {}
                }
            }
        }
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
