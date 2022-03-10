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
use libp2p::autonat;

#[tokio::main]
async fn main() {
    let args: Vec<String> = std::env::args().collect();
    let bytes: [u8; 32] = decode_sized(&args[1]).unwrap();
    let key = identity::ed25519::SecretKey::from_bytes(bytes).unwrap();
    let identity = identity::Keypair::Ed25519(key.into());
    
    let auto_nat =
        autonat::Behaviour::new(identity.public().to_peer_id(), autonat::Config::default());
    let ping = Ping::new(ping::Config::new().with_keep_alive(true));
    let rendezvous = rendezvous::server::Behaviour::new(rendezvous::server::Config::default());
    let identify = Identify::new(IdentifyConfig::new(
        "rendezvous-example/1.0.0".to_string(),
        identity.public(),
    ));
    println!("Peer id: {}", identity.public().to_peer_id());
    let mut swarm = Swarm::new(
        development_transport(identity.clone()).await.unwrap(),
        BootstrapBehaviour {
            identify,
            rendezvous,
            ping,
            auto_nat,
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
    AutoNat(autonat::Event),
}

#[derive(NetworkBehaviour)]
#[behaviour(event_process = true)]
#[behaviour(out_event = "BootstrapEvent")]
struct BootstrapBehaviour {
    identify: Identify,
    rendezvous: rendezvous::server::Behaviour,
    ping: Ping,
    auto_nat: autonat::Behaviour,
}

impl NetworkBehaviourEventProcess<IdentifyEvent> for BootstrapBehaviour {
    fn inject_event(&mut self, event: IdentifyEvent) {
        println!("{:?}", event);
    }
}
impl NetworkBehaviourEventProcess<autonat::Event> for BootstrapBehaviour {
    fn inject_event(&mut self, event: autonat::Event) {
        //println!("{:?}", event);
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

impl From<IdentifyEvent> for BootstrapEvent {
    fn from(v: IdentifyEvent) -> Self {
        Self::Identify(v)
    }
}

/*impl From<autonat::Event> for BootstrapEvent {
    fn from(v: autonat::Event) -> Self {
        Self::AutoNat(v)
    }
}*/