use std::{str::FromStr, sync::Arc};

use idp2p_common::{anyhow::Result, ed_secret::EdSecret};
use idp2p_core::protocol::{IdGossipMessage, IdGossipMessagePayload};
use libp2p::{
    core::{self, muxing::StreamMuxerBox, transport::Boxed},
    dns,
    gossipsub::{Gossipsub, GossipsubEvent},
    identify::{Identify, IdentifyConfig, IdentifyEvent},
    identity::{ed25519::SecretKey, Keypair},
    mdns::{Mdns, MdnsEvent},
    mplex, noise,
    ping::{self},
    relay::v2::relay::{self, Relay},
    swarm::SwarmBuilder,
    tcp, websocket, yamux, Multiaddr, NetworkBehaviour, PeerId, Swarm, Transport,
};

use crate::{
    gossip::build_gossipsub,
    store::{HandleGetResult, NodeStore},
};

#[derive(NetworkBehaviour)]
#[behaviour(out_event = "IdentityNodeEvent")]
pub struct IdentityNodeBehaviour {
    mdns: Mdns,
    gossipsub: Gossipsub,
    relay: Relay,
    ping: ping::Ping,
    identify: Identify,
    #[behaviour(ignore)]
    store: Arc<NodeStore>,
}

#[derive(Debug)]
pub enum IdentityNodeEvent {
    Gossipsub(GossipsubEvent),
    Mdns(MdnsEvent),
    Ping(ping::PingEvent),
    Identify(IdentifyEvent),
    Relay(relay::Event),
}

impl From<GossipsubEvent> for IdentityNodeEvent {
    fn from(event: GossipsubEvent) -> Self {
        IdentityNodeEvent::Gossipsub(event)
    }
}

impl From<MdnsEvent> for IdentityNodeEvent {
    fn from(event: MdnsEvent) -> Self {
        IdentityNodeEvent::Mdns(event)
    }
}

impl From<IdentifyEvent> for IdentityNodeEvent {
    fn from(event: IdentifyEvent) -> Self {
        IdentityNodeEvent::Identify(event)
    }
}

impl From<relay::Event> for IdentityNodeEvent {
    fn from(event: relay::Event) -> Self {
        IdentityNodeEvent::Relay(event)
    }
}

impl From<ping::PingEvent> for IdentityNodeEvent {
    fn from(event: ping::PingEvent) -> Self {
        IdentityNodeEvent::Ping(event)
    }
}

impl IdentityNodeBehaviour {
    pub async fn handle_gossip_event(&mut self, event: GossipsubEvent) -> Result<()> {
        if let GossipsubEvent::Message {
            propagation_source: _,
            message_id: _,
            message,
        } = event
        {
            let topic = message.topic.to_string();
            if idp2p_common::is_idp2p(&topic) {
                let message = IdGossipMessage::from_bytes(&message.data);
                match &message.payload {
                    IdGossipMessagePayload::Get => {
                        let get_result = self.store.handle_get(&topic).await?;
                        match get_result {
                            HandleGetResult::Publish(id) => {}
                            HandleGetResult::WaitAndPublish(duration) => {}
                        }
                    }
                    IdGossipMessagePayload::Post { digest, identity } => {
                        self.store.handle_post(digest, identity).await?;
                    }
                    IdGossipMessagePayload::Jwm { jwm } => {
                        //handle_jwm(&jwm, self);
                    }
                }
            }
        }
        Ok(())
    }
}

pub struct NodeOptions {
    port: u16,
    to_dial: Option<String>,
}

impl NodeOptions {
    pub fn new(to_dial: Option<String>) -> Self {
        Self {
            to_dial: to_dial,
            port: 43727,
        }
    }
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
    let id_store = Arc::new(NodeStore::new());
    let mut swarm = {
        let ping = ping::Ping::new(ping::Config::new().with_keep_alive(true));
        let identify = Identify::new(IdentifyConfig::new(
            "rendezvous-example/1.0.0".to_string(),
            local_key.public(),
        ));
        let relay = Relay::new(local_key.public().to_peer_id(), Default::default());
        let behaviour = IdentityNodeBehaviour {
            gossipsub: build_gossipsub(),
            mdns: Mdns::new(Default::default()).await?,
            relay,
            identify,
            ping,
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
