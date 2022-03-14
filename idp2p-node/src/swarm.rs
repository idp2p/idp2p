use idp2p_core::store::IdentityEvent;
use tokio::sync::mpsc::Sender;
use crate::behaviour::IdentityGossipBehaviour;
use idp2p_core::store::IdStore;
use idp2p_common::anyhow::Result;
use libp2p::Swarm;
use libp2p::{
    core,
    core::muxing::StreamMuxerBox,
    core::transport::Boxed,
    dns,
    gossipsub::{
        Gossipsub, GossipsubConfigBuilder, GossipsubMessage, MessageAuthenticity, MessageId,
        ValidationMode,
    },
    identify::{Identify, IdentifyConfig},
    identity, mplex, noise, rendezvous,
    swarm::SwarmBuilder,
    tcp, websocket, yamux, PeerId, Transport,
};
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::time::Duration;
use libp2p::relay::v2::relay::Relay;

pub struct SwarmOptions {
    pub port: u16,
    pub tx: Sender<IdentityEvent>
}

pub async fn build_transport(
    keypair: identity::Keypair,
) -> std::io::Result<Boxed<(PeerId, StreamMuxerBox)>> {
    let transport = {
        let tcp = tcp::TcpConfig::new().nodelay(true);
        let dns_tcp = dns::DnsConfig::system(tcp).await?;
        let ws_dns_tcp = websocket::WsConfig::new(dns_tcp.clone());
        dns_tcp.or_transport(ws_dns_tcp)
    };

    let noise_keys = noise::Keypair::<noise::X25519Spec>::new()
        .into_authentic(&keypair)
        .expect("Signing libp2p-noise static DH keypair failed.");

    Ok(transport
        .upgrade(core::upgrade::Version::V1)
        .authenticate(noise::NoiseConfig::xx(noise_keys).into_authenticated())
        .multiplex(core::upgrade::SelectUpgrade::new(
            yamux::YamuxConfig::default(),
            mplex::MplexConfig::default(),
        ))
        .timeout(std::time::Duration::from_secs(20))
        .boxed())
}

pub async fn create_swarm(options: SwarmOptions) -> Result<Swarm<IdentityGossipBehaviour>> {
    let local_key = identity::Keypair::generate_ed25519();
    let local_peer_id = PeerId::from(local_key.public());
    println!("Local peer id: {:?}", local_peer_id);
    let transport = build_transport(local_key.clone()).await?;

    let mut swarm = {
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
        let identify = Identify::new(IdentifyConfig::new(
            "rendezvous-example/1.0.0".to_string(),
            local_key.public(),
        ));
        let rendezvous = rendezvous::server::Behaviour::new(rendezvous::server::Config::default());
        let relay = Relay::new(local_key.public().to_peer_id(), Default::default());
        let id_store = IdStore::new(options.tx.clone());
        let behaviour = IdentityGossipBehaviour {
            identify: identify,
            rendezvous: rendezvous,
            gossipsub: gossipsub,
            relay: relay,
            id_store: id_store
        };
        let executor = Box::new(|fut| {
            tokio::spawn(fut);
        });
        SwarmBuilder::new(transport, behaviour, local_peer_id)
            .executor(executor)
            .build()
    };
    swarm.listen_on(format!("/ip4/0.0.0.0/tcp/{}", options.port).parse()?)?;
    Ok(swarm)
}
