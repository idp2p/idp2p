use crate::message::{IdentityMessage, IdentityMessagePayload};
use crate::store::IdEntry;
use crate::store::IdStore;
use idp2p_common::anyhow::Result;
use idp2p_common::serde_json;
use idp2p_core::did::Identity;
use libp2p::dcutr;
use libp2p::relay::v2::client::{self, Client};
use libp2p::{
    autonat,
    gossipsub::{Gossipsub, GossipsubEvent, IdentTopic},
    identify::{Identify, IdentifyEvent},
    multiaddr::Protocol,
    ping::{Ping, PingEvent},
    rendezvous,
    swarm::NetworkBehaviourEventProcess,
    Multiaddr, NetworkBehaviour,
};
use serde::{Deserialize, Serialize};

#[derive(NetworkBehaviour)]
#[behaviour(event_process = true)]
#[behaviour(out_event = "BootstrapEvent")]
pub struct IdentityGossipBehaviour {
    pub identify: Identify,
    pub auto_nat: autonat::Behaviour,
    pub rendezvous: rendezvous::client::Behaviour,
    pub ping: Ping,
    pub gossipsub: Gossipsub,
    pub relay_client: Client,
    pub dcutr: dcutr::behaviour::Behaviour,
    #[behaviour(ignore)]
    pub store: IdStore,
}

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub enum IdentityEvent {
    Discovered { addr: Multiaddr },
    Created { id: String },
    Updated { id: String },
    Requested { id: String },
    ReceivedJwm { id: String, jwm: String },
}

#[derive(Debug)]
pub enum BootstrapEvent {
    Rendezvous(rendezvous::client::Event),
    Ping(PingEvent),
    Identify(IdentifyEvent),
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
                let event = IdentityEvent::Requested { id: id.to_owned() };
                self.store.publish_event(event);
            } else {
                // to do()
            }
        }
        Ok(())
    }

    fn handle_post(&mut self, digest: &str, identity: &Identity) -> Result<()> {
        let current = self.store.get(&identity.id);
        match current {
            None => {
                identity.verify()?;
                let entry = IdEntry::from(identity.clone());
                self.store.put(&identity.id, entry);
                let event = IdentityEvent::Created {
                    id: identity.id.clone(),
                };
                self.store.publish_event(event);
            }
            Some(entry) => {
                if digest != entry.digest {
                    entry.did.is_next(identity.clone())?;
                    let new_entry = IdEntry {
                        did: identity.clone(),
                        ..entry
                    };
                    self.store.put(&identity.id, new_entry);
                }
                let event = IdentityEvent::Updated {
                    id: identity.id.clone(),
                };
                self.store.publish_event(event);
            }
        }
        Ok(())
    }
}

impl NetworkBehaviourEventProcess<GossipsubEvent> for IdentityGossipBehaviour {
    fn inject_event(&mut self, message: GossipsubEvent) {
        println!("Got message: {:?}", message);
        if let GossipsubEvent::Message {
            propagation_source: _,
            message_id: _,
            message,
        } = message
        {
            let topic = message.topic.to_string();
            let message: IdentityMessage = serde_json::from_slice(&message.data)
                .expect("Message is not well-formed. It should be json");
            let _ = match &message.payload {
                IdentityMessagePayload::Get => self.handle_get(&topic),
                IdentityMessagePayload::Post { digest, identity } => {
                    self.handle_post(digest, identity)
                }
                IdentityMessagePayload::Jwm { message } => {
                    let event = IdentityEvent::ReceivedJwm {
                        id: topic.to_owned(),
                        jwm: message.to_owned(),
                    };
                    self.store.publish_event(event);
                    Ok(())
                }
            };
        }
    }
}

impl NetworkBehaviourEventProcess<IdentifyEvent> for IdentityGossipBehaviour {
    fn inject_event(&mut self, event: IdentifyEvent) {
        if let IdentifyEvent::Received { peer_id, .. } = event {
            println!("Identify event: {}", peer_id);
            self.rendezvous.register(
                rendezvous::Namespace::from_static("rendezvous"),
                peer_id,
                None,
            );
        }
    }
}

impl NetworkBehaviourEventProcess<PingEvent> for IdentityGossipBehaviour {
    fn inject_event(&mut self, _: PingEvent) {}
}

impl NetworkBehaviourEventProcess<autonat::Event> for IdentityGossipBehaviour {
    fn inject_event(&mut self, _: autonat::Event) {}
}

impl NetworkBehaviourEventProcess<client::Event> for IdentityGossipBehaviour {
    fn inject_event(&mut self, event: client::Event) {
        println!("{:?}", event);
    }
}
impl NetworkBehaviourEventProcess<dcutr::behaviour::Event> for IdentityGossipBehaviour {
    fn inject_event(&mut self, event: dcutr::behaviour::Event) {
        println!("{:?}", event);
    }
}

impl NetworkBehaviourEventProcess<rendezvous::client::Event> for IdentityGossipBehaviour {
    fn inject_event(&mut self, event: rendezvous::client::Event) {
        if let rendezvous::client::Event::Discovered { registrations, .. } = event {
            for registration in registrations {
                for address in registration.record.addresses() {
                    let peer = registration.record.peer_id();
                    let p2p_suffix = Protocol::P2p(*peer.as_ref());
                    let address_with_p2p =
                        if !address.ends_with(&Multiaddr::empty().with(p2p_suffix.clone())) {
                            address.clone().with(p2p_suffix)
                        } else {
                            address.clone()
                        };
                    self.store.publish_event(IdentityEvent::Discovered {
                        addr: address_with_p2p,
                    });
                    println!("Peer {}, Addr: {}", peer, address);
                }
            }
        }
    }
}
