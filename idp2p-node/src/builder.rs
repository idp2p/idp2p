use idp2p_common::{anyhow::Result, ed_secret::EdSecret};
use libp2p::{
    core,
    core::muxing::StreamMuxerBox,
    core::transport::Boxed,
    dns,
    gossipsub::{
        Gossipsub, GossipsubConfigBuilder, GossipsubMessage, MessageAuthenticity, MessageId,
        ValidationMode,
    },
    identity::{ed25519::SecretKey, Keypair},
    mdns::Mdns,
    mplex, noise,
    swarm::SwarmBuilder,
    tcp, websocket, yamux, Multiaddr, PeerId, Swarm, Transport,
};
use std::hash::{Hash, Hasher};
use std::time::Duration;
use std::{collections::hash_map::DefaultHasher, str::FromStr, sync::Arc};
use tokio::sync::mpsc::Sender;

use crate::{behaviour::IdentityNodeBehaviour, id_store::IdNodeStore, IdentityStoreEvent};
pub struct NodeOptions {
    port: u16,
    to_dial: Option<String>,
    id_event_sender: Sender<IdentityStoreEvent>,
}

impl NodeOptions{
    pub fn new(tx: Sender<IdentityStoreEvent>, to_dial: Option<String>) -> Self{
        Self{
            id_event_sender: tx,
            to_dial: to_dial,
            port: 43727
        }
    }
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

pub async fn build_swarm(options: NodeOptions) -> Result<Swarm<IdentityNodeBehaviour>> {
    let secret_key = SecretKey::from_bytes(EdSecret::new().to_bytes())?;
    let local_key = Keypair::Ed25519(secret_key.into());
    let transport = build_transport(local_key.clone()).await;
    let id_store = Arc::new(IdNodeStore::new(options.id_event_sender.clone()));
    let mut swarm = {
        let behaviour = IdentityNodeBehaviour {
            gossipsub: build_gossipsub(),
            mdns: Mdns::new(Default::default()).await?,
            store: id_store,
        };
        let executor = Box::new(|fut| {
            tokio::spawn(fut);
        });
        SwarmBuilder::new(transport, behaviour, local_key.public().to_peer_id())
            .executor(executor)
            .build()
    };
    swarm.listen_on(format!("/ip4/0.0.0.0/tcp/{}", options.port).parse()?)?;
    if let Some(connect) = options.to_dial {
        let split: Vec<&str> = connect.split("/").collect();
        let to_dial = format!("/ip4/{}/tcp/{}", split[2], split[4]);
        let addr: Multiaddr = to_dial.parse().unwrap();
        let peer_id = PeerId::from_str(split[6])?;
        swarm.dial(addr)?;
        swarm.behaviour_mut().gossipsub.add_explicit_peer(&peer_id);
    }
    Ok(swarm)
}
