use crate::store::ClientStore;
use idp2p_core::store::IdStore;
use std::sync::Arc;
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
    pub identities: Arc<IdStore>,
    #[behaviour(ignore)]
    pub clients: Arc<ClientStore>,
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
