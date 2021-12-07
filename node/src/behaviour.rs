use crate::id_message::{IdentityCommand, IdentityMessage};
use core::did::Identity;
use libp2p::{
    gossipsub::{Gossipsub, GossipsubEvent, IdentTopic},
    mdns::{Mdns, MdnsEvent},
    swarm::NetworkBehaviourEventProcess,
    NetworkBehaviour,
};
use std::collections::HashMap;

#[derive(NetworkBehaviour)]
#[behaviour(event_process = true)]
pub struct IdentityGossipBehaviour {
    pub gossipsub: Gossipsub,
    pub mdns: Mdns,
    #[behaviour(ignore)]
    pub db: HashMap<String, String>,
}

fn handle_post(behaviour: &mut IdentityGossipBehaviour, topic: String, candidate: Identity) {
    let current = behaviour.db.get(&topic);
    match current {
        None => {
            behaviour.db.insert(
                topic.to_string(),
                serde_json::to_string(&candidate).unwrap(),
            );
            println!("Identity created...");
        }
        Some(c) => {
            let current_did: Identity = serde_json::from_str(c).unwrap();
            let r =current_did.is_next(candidate.clone());
            if r.is_ok() {
                behaviour.db.insert(
                    topic.to_string(),
                    serde_json::to_string(&candidate).unwrap(),
                );
                println!("Identity updated....");
            }else{
                println!("Error!!! {:?}", r);
            }
        }
    }
}

impl IdentityGossipBehaviour {
    pub fn publish(&mut self, topic: String, idm: IdentityMessage) {
        println!("Published topic: {}", topic.clone());
        let gossip_topic = IdentTopic::new(topic.clone());
        let _ = self.gossipsub.publish(
            gossip_topic,
            serde_json::to_string(&idm).unwrap().as_bytes(),
        ).unwrap();
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
            let id_mes: IdentityMessage = serde_json::from_slice(&message.data).unwrap();
            println!("Received topic: {}", topic);
            match id_mes.command {
                IdentityCommand::Get => {
                    let identity: Identity =
                        serde_json::from_str(&self.db.get(&topic).unwrap()).unwrap();
                    self.publish(
                        topic.clone(),
                        IdentityMessage::new(IdentityCommand::Post(identity)),
                    );
                }
                IdentityCommand::Post(identity) => handle_post(self, topic.clone(), identity),
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
