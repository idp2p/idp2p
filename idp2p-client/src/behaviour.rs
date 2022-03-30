use idp2p_core::message::Publisher;
use idp2p_core::message::{IdentityMessage, IdentityMessagePayload};
use idp2p_core::store::IdStore;
use std::sync::Arc;
use libp2p::{
    gossipsub::{Gossipsub, GossipsubEvent},
    mdns::{Mdns, MdnsEvent},
    NetworkBehaviour,
};

#[derive(NetworkBehaviour)]
#[behaviour(out_event = "IdentityClientEvent")]
pub struct IdentityClientBehaviour {
    pub mdns: Mdns,
    pub gossipsub: Gossipsub,
    #[behaviour(ignore)]
    pub id_store: Arc<IdStore>,
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

impl IdentityClientBehaviour {
    pub async fn handle_gossipsub_event(&mut self, event: GossipsubEvent){
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
                        let mes = self.id_store.handle_get(&topic);
                        if let Some(mes) = mes {
                            self.gossipsub.publish_msg(mes).unwrap();
                        }
                    }
                    IdentityMessagePayload::Post { digest, identity } => {
                        self.id_store.handle_post(digest, identity).unwrap();
                    }
                    _ => {}
                }
            }
        }
    }

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