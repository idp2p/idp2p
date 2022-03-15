use crate::IdentityEvent;
use tokio::sync::mpsc::Sender;
use idp2p_core::did::Identity;
use crate::message::{IdentityMessage, IdentityMessagePayload};
use crate::store::IdStore;
use idp2p_common::anyhow::Result;

use libp2p::{
    mdns::{Mdns, MdnsEvent},
    identify::{Identify, IdentifyConfig, IdentifyEvent},
    swarm::{NetworkBehaviourEventProcess, Swarm, SwarmBuilder},
    NetworkBehaviour,
    rendezvous,
    core,
    core::muxing::StreamMuxerBox,
    core::transport::Boxed,
    dns,
    gossipsub::{
        Gossipsub, GossipsubConfigBuilder, GossipsubMessage, MessageAuthenticity, MessageId,
        ValidationMode,GossipsubEvent
    },
    identity::Keypair,
    mplex, noise, tcp, websocket, yamux, PeerId, Transport,
};
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::time::Duration;

#[derive(NetworkBehaviour)]
#[behaviour(event_process = true)]
pub struct IdentityNodeBehaviour {
    pub mdns: Mdns,
    pub identify: Identify,
    pub gossipsub: Gossipsub,
    pub rendezvous: rendezvous::server::Behaviour,
    #[behaviour(ignore)]
    pub id_store: IdStore,
}

pub struct SwarmOptions {
    pub port: u16,
    pub owner: Identity,
    pub event_sender: Sender<IdentityEvent>
}

async fn build_transport(local_key: Keypair) -> Boxed<(PeerId, StreamMuxerBox)> {
    let local_peer_id = PeerId::from(local_key.public());
    println!("Local peer id: {:?}", local_peer_id);
    let transport = {
        let tcp = tcp::TcpConfig::new().nodelay(true);
        let dns_tcp = dns::DnsConfig::system(tcp).await.unwrap();
        let ws_dns_tcp = websocket::WsConfig::new(dns_tcp.clone());
        dns_tcp.or_transport(ws_dns_tcp)
    };

    let noise_keys = noise::Keypair::<noise::X25519Spec>::new()
        .into_authentic(&local_key)
        .expect("Signing libp2p-noise static DH keypair failed.");

    let boxed = transport
        .upgrade(core::upgrade::Version::V1)
        .authenticate(noise::NoiseConfig::xx(noise_keys).into_authenticated())
        .multiplex(core::upgrade::SelectUpgrade::new(
            yamux::YamuxConfig::default(),
            mplex::MplexConfig::default(),
        ))
        .timeout(std::time::Duration::from_secs(20))
        .boxed();
    boxed
}

fn build_gossipsub() -> Gossipsub {
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
    gossipsub
}

pub async fn build_swarm(options: SwarmOptions) -> Result<Swarm<IdentityNodeBehaviour>> {
    let local_key = Keypair::generate_ed25519();
    let transport = build_transport(local_key.clone()).await;
    let mut swarm = {
        let mdns = Mdns::new(Default::default()).await?;
        let identify = Identify::new(IdentifyConfig::new(
            "rendezvous-example/1.0.0".to_string(),
            local_key.public(),
        ));
        let rendezvous = rendezvous::server::Behaviour::new(rendezvous::server::Config::default());
        let id_store = IdStore::new(options.event_sender.clone(), options.owner);
        let behaviour = IdentityNodeBehaviour {
            mdns: mdns,
            identify: identify,
            gossipsub: build_gossipsub(),
            rendezvous: rendezvous,
            id_store: id_store
        };
        let executor = Box::new(|fut| {
            tokio::spawn(fut);
        });
        SwarmBuilder::new(transport, behaviour, local_key.public().to_peer_id())
            .executor(executor)
            .build()
    };
    swarm.listen_on(format!("/ip4/0.0.0.0/tcp/{}", options.port).parse()?)?;
    Ok(swarm)
}

impl NetworkBehaviourEventProcess<GossipsubEvent> for IdentityNodeBehaviour {
    fn inject_event(&mut self, message: GossipsubEvent) {
        if let GossipsubEvent::Message {
            propagation_source: _,
            message_id: _,
            message,
        } = message
        {
            let topic = message.topic.to_string();
            if idp2p_common::is_idp2p(&topic) {
                let message = IdentityMessage::from_bytes(&message.data);
                match &message.payload {
                    IdentityMessagePayload::Get => {
                        self.id_store.handle_get(&topic);
                    }
                    IdentityMessagePayload::Post { digest, identity } => {
                        self.id_store.handle_post(digest, identity).unwrap();
                    }
                    _ => {}
                }
            }
        }
    }
}

impl NetworkBehaviourEventProcess<IdentifyEvent> for IdentityNodeBehaviour {
    fn inject_event(&mut self, event: IdentifyEvent) {
        if let IdentifyEvent::Received { peer_id, .. } = event {
            println!("Identify event: {}", peer_id);
        }
    }
}

impl NetworkBehaviourEventProcess<rendezvous::server::Event> for IdentityNodeBehaviour {
    fn inject_event(&mut self, e: rendezvous::server::Event) {
        println!("{:?}", e);
    }
}

impl NetworkBehaviourEventProcess<MdnsEvent> for IdentityNodeBehaviour {
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