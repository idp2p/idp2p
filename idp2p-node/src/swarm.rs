use crate::behaviour::IdentityGossipBehaviour;
use crate::store::IdStore;
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
    identity, mplex, noise, ping, rendezvous,
    swarm::SwarmBuilder,
    tcp, websocket, yamux, PeerId, Transport,
};
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::time::Duration;
use libp2p::relay::v2::client::{self, Client};
use libp2p::dcutr;

pub struct SwarmOptions {
    pub addr: String,
    pub port: u16,
    pub store: IdStore,
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
    let (relay_transport, client) = Client::new_transport_and_behaviour(local_peer_id);
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
        let rendezvous = rendezvous::client::Behaviour::new(local_key.clone());
        let ping = ping::Ping::new(ping::Config::new().with_keep_alive(true));
        
        let dcutr = dcutr::behaviour::Behaviour::new();
        let behaviour = IdentityGossipBehaviour {
            identify: identify,
            rendezvous: rendezvous,
            ping: ping,
            gossipsub: gossipsub,
            store: options.store,
            dcutr: dcutr,
            relay_client: client
        };
        let executor = Box::new(|fut| {
            tokio::spawn(fut);
        });
        SwarmBuilder::new(transport, behaviour, local_peer_id)
            .executor(executor)
            .build()
    };
    swarm.listen_on(format!("/ip4/{}/tcp/{}", options.addr, options.port).parse()?)?;
    Ok(swarm)
}
