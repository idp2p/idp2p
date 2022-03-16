use crate::message::{IdentityMessage, IdentityMessagePayload};
use crate::store::IdStore;
use crate::IdentityEvent;
use idp2p_common::anyhow::Result;
use idp2p_core::did::Identity;
use tokio::sync::mpsc::Sender;

use libp2p::{
    core::muxing::StreamMuxerBox,
    core::transport::Boxed,
    dns,
    gossipsub::{
        Gossipsub, GossipsubConfigBuilder, GossipsubEvent, GossipsubMessage, IdentTopic,
        MessageAuthenticity, MessageId, ValidationMode,
    },
    identity::Keypair,
    mdns::{Mdns, MdnsEvent},
    mplex, noise, core,
    swarm::{Swarm, SwarmBuilder},
    tcp, websocket, yamux, NetworkBehaviour, PeerId, Transport,
};
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::time::Duration;

#[derive(NetworkBehaviour)]
#[behaviour(out_event = "IdentityNodeEvent")]
pub struct IdentityNodeBehaviour {
    mdns: Mdns,
    pub gossipsub: Gossipsub,
    #[behaviour(ignore)]
    pub id_store: IdStore,
}

impl IdentityNodeBehaviour {
    pub async fn handle_gossipevent(&mut self, event: GossipsubEvent) {
        if let GossipsubEvent::Message {
            propagation_source: _,
            message_id: _,
            message,
        } = event
        {
            let topic = message.topic.to_string();
            if idp2p_common::is_idp2p(&topic) {
                let message = IdentityMessage::from_bytes(&message.data);
                match &message.payload {
                    IdentityMessagePayload::Get => {
                        self.id_store.handle_get(&topic).await;
                    }
                    IdentityMessagePayload::Post { digest, identity } => {
                        self.id_store.handle_post(digest, identity).await.unwrap();
                    }
                    IdentityMessagePayload::Jwm { message } => {
                        self.id_store.handle_jwm(&topic, message).await;
                    }
                }
            }
        }
    }

    pub fn handle_mdnsevent(&mut self, event: MdnsEvent) {
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

    pub fn post(&mut self, id: &str) {
        let did = self.id_store.get_did(id);
        let gossip_topic = IdentTopic::new(id);
        let message = IdentityMessage::new_post(did);
        let json_str = idp2p_common::serde_json::to_string(&message).unwrap();
        let result = self.gossipsub.publish(gossip_topic, json_str.as_bytes());
        match result {
            Ok(_) => println!("Published id: {}", id),
            Err(e) => println!("Publish error, {:?}", e),
        }
    }
}

#[derive(Debug)]
pub enum IdentityNodeEvent {
    Mdns(MdnsEvent),
    Gossipsub(GossipsubEvent),
}

impl From<MdnsEvent> for IdentityNodeEvent {
    fn from(event: MdnsEvent) -> Self {
        IdentityNodeEvent::Mdns(event)
    }
}

impl From<GossipsubEvent> for IdentityNodeEvent {
    fn from(event: GossipsubEvent) -> Self {
        IdentityNodeEvent::Gossipsub(event)
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

pub struct SwarmOptions {
    pub port: u16,
    pub owner: Identity,
    pub event_sender: Sender<IdentityEvent>,
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

pub async fn build_swarm(options: SwarmOptions) -> Result<Swarm<IdentityNodeBehaviour>> {
    let local_key = Keypair::generate_ed25519();
    let transport = build_transport(local_key.clone()).await;
    let mut swarm = {
        let mdns = Mdns::new(Default::default()).await?;
        let id_store = IdStore::new(options.event_sender.clone(), options.owner);
        let behaviour = IdentityNodeBehaviour {
            mdns: mdns,
            gossipsub: build_gossipsub(),
            id_store: id_store,
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
