use crate::message::IdentityMessage;
use crate::store::IdStore;
use idp2p_common::anyhow::Result;
use idp2p_common::serde_json;
use idp2p_core::did::Identity;
use libp2p::{
    gossipsub::{Gossipsub, GossipsubEvent, IdentTopic},
    mdns::{Mdns, MdnsEvent},
    swarm::NetworkBehaviourEventProcess,
    NetworkBehaviour,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tokio::sync::mpsc::Sender;

#[derive(NetworkBehaviour)]
#[behaviour(event_process = true)]
pub struct IdentityGossipBehaviour {
    pub gossipsub: Gossipsub,
    pub mdns: Mdns,
    #[behaviour(ignore)]
    pub identities: HashMap<String, String>,
    #[behaviour(ignore)]
    pub sender: Sender<IdentityGossipEvent>,
}

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub struct IdentityGossipEvent {
    pub topic: String,
    pub message: IdentityMessage,
}

impl IdentityGossipBehaviour {
    pub fn publish(&mut self, id: String, mes: IdentityMessage) {
        let gossip_topic = IdentTopic::new(id.clone());
        let json_str = serde_json::to_string(&mes).unwrap();
        let result = self.gossipsub.publish(gossip_topic, json_str.as_bytes());
        match result {
            Ok(_) => println!("Published id: {}", id.clone()),
            Err(e) => println!("Publish error, {:?}", e),
        }
    }

    pub fn create(&mut self, did: Identity, store: impl IdStore) -> Result<()> {
        let gossipsub_topic = IdentTopic::new(did.id.clone());
        store.put(&did.id.clone(), did.clone());
        self.gossipsub.subscribe(&gossipsub_topic)?;
        Ok(())
    }
}

impl NetworkBehaviourEventProcess<GossipsubEvent> for IdentityGossipBehaviour {
    fn inject_event(&mut self, message: GossipsubEvent) {
        if let GossipsubEvent::Message {
            propagation_source: _,
            message_id: _,
            message,
        } = message
        {
            let topic = message.topic.to_string();
            let message: IdentityMessage =
                serde_json::from_slice(&message.data).expect("Message is not well-formed");
            let e = IdentityGossipEvent { topic, message };
            self.sender.try_send(e).expect("Couldn't send event");
        }
    }
}

impl NetworkBehaviourEventProcess<MdnsEvent> for IdentityGossipBehaviour {
    fn inject_event(&mut self, event: MdnsEvent) {
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
