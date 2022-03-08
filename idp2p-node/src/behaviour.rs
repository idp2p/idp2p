use crate::message::{IdentityMessage, IdentityMessagePayload};
use crate::store::IdEntry;
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

#[derive(NetworkBehaviour)]
#[behaviour(event_process = true)]
pub struct IdentityGossipBehaviour {
    pub gossipsub: Gossipsub,
    pub mdns: Mdns,
    #[behaviour(ignore)]
    pub store: IdStore,
}

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub enum IdentityEvent {
    Skipped,
    Created { id: String },
    Updated { id: String },
    Requested { id: String },
    ReceivedJwm { id: String, jwm: String },
}

impl IdentityGossipBehaviour {
    pub fn publish(&mut self, id: String, mes: IdentityMessage) -> Result<()> {
        let gossip_topic = IdentTopic::new(id.clone());
        let json_str = serde_json::to_string(&mes).unwrap();
        let result = self.gossipsub.publish(gossip_topic, json_str.as_bytes());
        match result {
            Ok(_) => println!("Published id: {}", id.clone()),
            Err(e) => println!("Publish error, {:?}", e),
        }
        Ok(())
    }

    pub fn subscribe(&mut self, id: String) -> Result<()> {
        let gossip_topic = IdentTopic::new(id.clone());
        self.gossipsub.subscribe(&gossip_topic)?;
        Ok(())
    }

    fn handle_get(&mut self, id: &str) -> Result<()> {
        if let Some(entry) = self.store.get(id) {
            if entry.is_hosted {
                let payload = IdentityMessagePayload::Post {
                    digest: entry.did.get_digest(),
                    identity: entry.did.clone(),
                };
                let mes = IdentityMessage::new(payload);
                self.publish(entry.did.id.clone(), mes)?;
            } else {
                // to do
            }
        }
        Ok(())
    }

    fn handle_post(&mut self, digest: &str, identity: &Identity) -> Result<()> {
        let current = self.store.get(&identity.id);
        match current {
            None => {
                identity.verify()?;
                /*let entry = IdEntry{
                    di
                };
                self.store.put(&identity.id, identity.clone());*/
            }
            Some(entry) => {
                /*if digest == current_digest {
                    return Ok(IdentityEvent::Skipped);
                }
                let current_did: Identity = self.store.get(&identity.id).unwrap();
                current_did.is_next(identity.clone())?;
                self.store.put(&identity.id, identity.clone());
                return Ok(IdentityEvent::Updated {
                    id: identity.id.clone(),
                });*/
            }
        }
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
            let r = match &message.payload {
                IdentityMessagePayload::Get => self.handle_get(&topic),
                IdentityMessagePayload::Post { digest, identity } => {
                    self.handle_post(digest, identity)
                }
                IdentityMessagePayload::Jwm { message } => {
                    Ok(())
                    /*let event = IdentityEvent::ReceivedJwm {
                        id: topic.to_owned(),
                        jwm: message.to_owned(),
                    };
                    self.store
                        .shared
                        .sender
                        .try_send(event)
                        .expect("Couldn't send event");*/
                }
            };
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
