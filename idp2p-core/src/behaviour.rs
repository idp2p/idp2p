use std::sync::Arc;

use crate::{
    message::{IdentityMessage, IdentityMessagePayload},
    store::IdStore,
};
use async_trait::async_trait;
use libp2p::{
    gossipsub::{Gossipsub, GossipsubEvent},
    mdns::{Mdns, MdnsEvent},
};

#[async_trait(?Send)]
pub trait IdGossipBehaviour {
    async fn handle_event(&mut self, event: GossipsubEvent, id_store: Arc<IdStore>);
}

pub trait IdMdnsBehaviour {
    fn handle_event(&mut self, mdns: Mdns, event: MdnsEvent);
}

#[async_trait(?Send)]
impl IdGossipBehaviour for Gossipsub {
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
                        //self.id_store.handle_post(digest, identity).await;
                    }
                    _ => {}
                }
            }
        }
    }
}

impl IdMdnsBehaviour for Gossipsub {
    fn handle_event(&mut self, mdns: Mdns, event: MdnsEvent) {
        match event {
            MdnsEvent::Discovered(list) => {
                for (peer, _) in list {
                    self.add_explicit_peer(&peer);
                }
            }
            MdnsEvent::Expired(list) => {
                for (peer, _) in list {
                    if !mdns.has_node(&peer) {
                        self.remove_explicit_peer(&peer);
                    }
                }
            }
        }
    }
}
