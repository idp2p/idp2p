use crate::file_store::FileStore;
use crate::id_message::{IdentityMessage, IdentityMessageType};
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

    fn handle_post(&mut self, candidate_digest: &str, candidate: Identity) -> Result<PostResult> {
        let current = self.identities.get(&candidate.id);
        match current {
            None => {
                candidate.verify()?;
                self.save(candidate.clone());
                Ok(PostResult::IdentityCreated)
            }
            Some(digest) => {
                if digest == candidate_digest {
                    return Ok(PostResult::Skipped);
                }
                let current_did: Identity = FileStore.get("identities", &candidate.id).unwrap();
                current_did.is_next(candidate.clone())?;
                self.save(candidate.clone());
                Ok(PostResult::IdentityUpdated)
            }
        }
    }

    fn save(&mut self, candidate: Identity) {
        self.identities
            .insert(candidate.id.clone(), candidate.get_digest());
        FileStore.put("identities", &candidate.id, candidate.clone());
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
            let id = message.topic.to_string();
            let id_mes: IdentityMessage = serde_json::from_slice(&message.data).unwrap();
            match id_mes.message {
                IdentityMessageType::Get => {
                    let identity: Identity = FileStore.get("identities", &id).unwrap();
                    let post = IdentityMessageType::Post {
                        digest: identity.get_digest(),
                        identity: identity.clone(),
                    };
                    self.publish(id.clone(), IdentityMessage::new(post));
                }
                IdentityMessageType::Post { digest, identity } => {
                    let result = self.handle_post(&digest, identity);
                    match result {
                        Ok(result) => match result {
                            PostResult::IdentityCreated => println!("Identity created {}", &id),
                            PostResult::IdentityUpdated => println!("Identity updated {}", &id),
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
