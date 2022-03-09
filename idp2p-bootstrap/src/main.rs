use crate::ping::PingEvent;
use std::collections::hash_map::DefaultHasher;
use libp2p::core::identity;
use libp2p::core::PeerId;
use libp2p::futures::StreamExt;
use libp2p::gossipsub::{
    Gossipsub, GossipsubConfigBuilder, GossipsubMessage, MessageAuthenticity, MessageId,
    ValidationMode, GossipsubEvent
};
use libp2p::identify::{Identify, IdentifyEvent, IdentifyConfig};
use libp2p::ping;
use libp2p::ping::Ping;
use libp2p::swarm::{Swarm, SwarmEvent};
use libp2p::NetworkBehaviour;
use libp2p::{development_transport, rendezvous};
use std::time::Duration;
use std::hash::{Hash, Hasher};
use libp2p::swarm::NetworkBehaviourEventProcess;
use dotenv::dotenv;

#[tokio::main]
async fn main() {
    dotenv().ok();
    let bytes = [0u8; 32];
    let key = identity::ed25519::SecretKey::from_bytes(bytes).unwrap();
    let identity = identity::Keypair::Ed25519(key.into());
    let message_id_fn = |message: &GossipsubMessage| {
        let mut s = DefaultHasher::new();
        message.data.hash(&mut s);
        MessageId::from(s.finish().to_string())
    };
    let gossipsub_config = GossipsubConfigBuilder::default()
        .heartbeat_interval(Duration::from_secs(10))
        .validation_mode(ValidationMode::Anonymous)
        .message_id_fn(message_id_fn)
        .build()
        .expect("Valid config");

    let gossipsub_result = Gossipsub::new(MessageAuthenticity::Anonymous, gossipsub_config);
    let gossipsub = gossipsub_result.expect("Correct configuration");
    let mut swarm = Swarm::new(
        development_transport(identity.clone()).await.unwrap(),
        BootstrapBehaviour {
            identify: Identify::new(IdentifyConfig::new(
                "rendezvous-example/1.0.0".to_string(),
                identity.public(),
            )),
            rendezvous: rendezvous::server::Behaviour::new(rendezvous::server::Config::default()),
            ping: Ping::new(ping::Config::new().with_keep_alive(true)),
            gossipsub: gossipsub
        },
        PeerId::from(identity.public()),
    );
    let addr = "/ip4/0.0.0.0/tcp/43727";
    swarm.listen_on(addr.parse().unwrap()).unwrap();

    while let Some(event) = swarm.next().await {
        match event {
            SwarmEvent::ConnectionEstablished { peer_id, .. } => {
                //println!("Connected to {}", peer_id);
            }
            SwarmEvent::ConnectionClosed { peer_id, .. } => {
                //println!("Disconnected from {}", peer_id);
            }
            /*SwarmEvent::Behaviour(BootstrapEvent::Rendezvous(
                rendezvous::server::Event::PeerRegistered { peer, registration },
            )) => {
                println!(
                    "Peer {} registered for namespace '{}'",
                    peer, registration.namespace
                );
            }
            SwarmEvent::Behaviour(BootstrapEvent::Rendezvous(
                rendezvous::server::Event::DiscoverServed {
                    enquirer,
                    registrations,
                },
            )) => {
                println!(
                    "Served peer {} with {} registrations",
                    enquirer,
                    registrations.len()
                );
            }*/
            other => {
                //println!("Unhandled {:?}", other);
            }
        }
    }
}

#[derive(Debug)]
pub enum BootstrapEvent {
    Rendezvous(rendezvous::server::Event),
    Ping(PingEvent),
    Identify(IdentifyEvent),
}

#[derive(NetworkBehaviour)]
#[behaviour(event_process = true)]
#[behaviour(out_event = "BootstrapEvent")]
struct BootstrapBehaviour {
    identify: Identify,
    rendezvous: rendezvous::server::Behaviour,
    ping: Ping,
    gossipsub: Gossipsub,
}

impl NetworkBehaviourEventProcess<GossipsubEvent> for BootstrapBehaviour {
    fn inject_event(&mut self, message: GossipsubEvent) {
        println!("Got message: {:?}", message);
        if let GossipsubEvent::Message {
            propagation_source: _,
            message_id: _,
            message,
        } = message
        {
            println!("{:?}", message);
        }
    }
}

impl NetworkBehaviourEventProcess<IdentifyEvent> for BootstrapBehaviour {
    fn inject_event(&mut self, event: IdentifyEvent) {
        println!("{:?}", event);
    }
}

impl NetworkBehaviourEventProcess<PingEvent> for BootstrapBehaviour {
    fn inject_event(&mut self, _: PingEvent) {
        //println!("{:?}", e);
    }
}

impl NetworkBehaviourEventProcess<rendezvous::server::Event> for BootstrapBehaviour {
    fn inject_event(&mut self, e: rendezvous::server::Event) {
        println!("{:?}", e);
    }
}