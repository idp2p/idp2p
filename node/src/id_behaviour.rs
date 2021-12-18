use crate::id_message::{IdentityMessageType, IdentityMessage};
use anyhow::Result;
use idp2p_core::did::Identity;
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
    pub identities: HashMap<String, String>,
}

enum PostResult {
    IdentityCreated,
    IdentityUpdated,
    Skipped,
}

fn handle_post(
    behaviour: &mut IdentityGossipBehaviour,
    topic: String,
    candidate: Identity,
) -> Result<PostResult> {
    let current = behaviour.identities.get(&topic);
    match current {
        None => {
            behaviour.identities.insert(
                topic.to_string(),
                candidate.get_digest(),
            );
            Ok(PostResult::IdentityCreated)
        }
        Some(c) => {
            let current_did: Identity = serde_json::from_str(c)?;
            if current_did.get_digest() == candidate.get_digest() {
                return Ok(PostResult::Skipped);
            }
            current_did.is_next(candidate.clone())?;
            behaviour.identities.insert(
                topic.to_string(),
                serde_json::to_string(&candidate).unwrap(),
            );
            Ok(PostResult::IdentityUpdated)
        }
    }
}

impl IdentityGossipBehaviour {
    pub fn publish(&mut self, topic: String, idm: IdentityMessage) {
        let gossip_topic = IdentTopic::new(topic.clone());
        let json_str = serde_json::to_string(&idm).unwrap();
        let result = self.gossipsub.publish(gossip_topic, json_str.as_bytes());
        match result {
            Ok(_) => println!("Published id: {}", topic.clone()),
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
            let id_mes: IdentityMessage = serde_json::from_slice(&message.data).unwrap();
            match id_mes.message {
                IdentityMessageType::Get => {
                    let identity: Identity =
                        serde_json::from_str(&self.identities.get(&topic).unwrap()).unwrap();
                    self.publish(
                        topic.clone(),
                        IdentityMessage::new(IdentityMessageType::Post(identity)),
                    );
                }
                IdentityMessageType::Post(identity) => {
                    let r = handle_post(self, topic.clone(), identity);
                    match r {
                        Ok(r) => match r {
                            PostResult::IdentityCreated => println!("Identity created {}", &topic),
                            PostResult::IdentityUpdated => println!("Identity updated {}", &topic),
                            _ => {}
                        },
                        Err(e) => println!("Error {}", e),
                    }
                }
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
