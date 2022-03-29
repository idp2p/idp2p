use libp2p::{
    core,
    core::muxing::StreamMuxerBox,
    core::transport::Boxed,
    dns,
    gossipsub::{
        Gossipsub, GossipsubConfigBuilder, GossipsubMessage, MessageAuthenticity, MessageId,
        ValidationMode,
    },
    identity::Keypair,
    mplex, noise, tcp, websocket, yamux, PeerId, Transport,
};
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::time::Duration;

pub(crate) fn build_gossipsub() -> Gossipsub {
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

pub(crate) async fn build_transport(local_key: Keypair) -> Boxed<(PeerId, StreamMuxerBox)> {
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

