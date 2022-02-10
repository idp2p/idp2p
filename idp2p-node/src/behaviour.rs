use crate::message::IdentityMessage;
use libp2p::{
    gossipsub::{Gossipsub, GossipsubEvent, IdentTopic},
    mdns::{Mdns, MdnsEvent},
    swarm::NetworkBehaviourEventProcess,
    NetworkBehaviour,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use idp2p_common::serde_json;

#[derive(NetworkBehaviour)]
#[behaviour(event_process = true)]
pub struct IdentityGossipBehaviour {
    pub gossipsub: Gossipsub,
    pub mdns: Mdns,
    #[behaviour(ignore)]
    pub identities: HashMap<String, String>,
    #[behaviour(ignore)]
    pub sender: tokio::sync::mpsc::Sender<IdentityGossipEvent>,
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
            let message: IdentityMessage = serde_json::from_slice(&message.data).unwrap();
            self.sender.try_send(IdentityGossipEvent { topic, message });
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
