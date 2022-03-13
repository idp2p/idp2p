use crate::ping::PingEvent;
use idp2p_common::decode_sized;
use libp2p::core::identity;
use libp2p::core::PeerId;
use libp2p::futures::StreamExt;
use libp2p::identify::{Identify, IdentifyConfig, IdentifyEvent};
use libp2p::ping;
use libp2p::ping::Ping;
use libp2p::swarm::NetworkBehaviourEventProcess;
use libp2p::swarm::{Swarm, SwarmEvent};
use libp2p::NetworkBehaviour;
use libp2p::{development_transport, rendezvous};
use libp2p::relay::v2::relay::{self, Relay};
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::time::Duration;
use libp2p::gossipsub::{
    Gossipsub, GossipsubConfigBuilder, GossipsubEvent, GossipsubMessage, MessageAuthenticity,
    MessageId, ValidationMode,
};

#[tokio::main]
async fn main() {
    let args: Vec<String> = std::env::args().collect();
    let bytes: [u8; 32] = decode_sized(&args[1]).unwrap();
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
    let ping = Ping::new(ping::Config::new().with_keep_alive(true));
    let rendezvous = rendezvous::server::Behaviour::new(rendezvous::server::Config::default());
    let identify = Identify::new(IdentifyConfig::new(
        "rendezvous-example/1.0.0".to_string(),
        identity.public(),
    ));
    let relay = Relay::new(identity.public().to_peer_id(), Default::default());
    println!("Peer id: {}", identity.public().to_peer_id());
    let mut swarm = Swarm::new(
        development_transport(identity.clone()).await.unwrap(),
        BootstrapBehaviour {
            identify,
            rendezvous,
            ping,
            relay,
            gossipsub,
        },
        PeerId::from(identity.public()),
    );
    let addr = "/ip4/0.0.0.0/tcp/43727";
    swarm.listen_on(addr.parse().unwrap()).unwrap();
    while let Some(event) = swarm.next().await {
        match event {
            SwarmEvent::NewListenAddr { address, .. } => {
                println!("Listening on {:?}", address);
            }
            SwarmEvent::ConnectionEstablished { peer_id, .. } => {
                println!("Connected to {}", peer_id);
            }
            SwarmEvent::ConnectionClosed { peer_id, .. } => {
                println!("Disconnected from {}", peer_id);
            }
            _ => {}
        }
    }
}

#[derive(Debug)]
pub enum BootstrapEvent {
    Rendezvous(rendezvous::server::Event),
    Ping(PingEvent),
    Identify(IdentifyEvent),
    Relay(relay::Event)
}

#[derive(NetworkBehaviour)]
#[behaviour(event_process = true)]
#[behaviour(out_event = "BootstrapEvent")]
struct BootstrapBehaviour {
    identify: Identify,
    rendezvous: rendezvous::server::Behaviour,
    ping: Ping,
    relay: Relay,
    gossipsub: Gossipsub,
}

impl NetworkBehaviourEventProcess<IdentifyEvent> for BootstrapBehaviour {
    fn inject_event(&mut self, event: IdentifyEvent) {
        println!("{:?}", event);
    }
}

impl NetworkBehaviourEventProcess<relay::Event> for BootstrapBehaviour {
    fn inject_event(&mut self, event: relay::Event) {
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

impl NetworkBehaviourEventProcess<GossipsubEvent> for BootstrapBehaviour {
    fn inject_event(&mut self, e: GossipsubEvent) {
        println!("{:?}", e);
    }
}

