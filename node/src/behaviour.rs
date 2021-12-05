use libp2p::{
    gossipsub::{Gossipsub, GossipsubEvent, IdentTopic},
    mdns::{Mdns, MdnsEvent},
    swarm::NetworkBehaviourEventProcess,
    NetworkBehaviour,
};
use std::collections::HashMap;
use std::fs::OpenOptions;

#[derive(NetworkBehaviour)]
#[behaviour(event_process = true)]
pub struct IdentityGossipBehaviour {
    pub gossipsub: Gossipsub,
    pub mdns: Mdns,
    #[behaviour(ignore)]
    pub db: HashMap<String, String>,
}

fn handle_get(behaviour: &mut IdentityGossipBehaviour, topic: String) {
    let current = behaviour.db.get(&topic).unwrap();
    let post_data = format!("post {}", current.clone());
    let gossip_topic = IdentTopic::new(topic);
    let _ = behaviour
        .gossipsub
        .publish(gossip_topic, post_data.as_bytes());
}

fn handle_post(behaviour: &mut IdentityGossipBehaviour, topic: String, content: String) {
    let current = behaviour.db.get(&topic);
    match current {
        None => {
            behaviour.db.insert(topic.to_string(), content.to_string());
        }
        Some(c) => {
            if content.len() > c.len() {
                behaviour.db.insert(topic.to_string(), content.to_string());
            }
        }
    }
}

impl NetworkBehaviourEventProcess<GossipsubEvent> for IdentityGossipBehaviour {
    fn inject_event(&mut self, message: GossipsubEvent) {
        println!("source: {:?}", message);
        if let GossipsubEvent::Message {
            propagation_source: _,
            message_id: _,
            message,
        } = message
        {
            let topic = message.topic.to_string();
            let data = String::from_utf8_lossy(&message.data);
            let split = data.split(" ");
            let input: Vec<&str> = split.collect();
            let command = input[0];
            match command {
                "get" => {
                    handle_get(self, topic.clone());
                }
                "post" => {
                    handle_post(self, topic.clone(), input[1].to_string());
                }
                _ => panic!(""),
            }
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
