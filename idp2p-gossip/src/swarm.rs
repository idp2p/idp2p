use libp2p::{
    gossipsub::{Gossipsub, GossipsubEvent},
    mdns::{Mdns, MdnsEvent},
    NetworkBehaviour,
};

#[derive(NetworkBehaviour)]
#[behaviour(out_event = "Idp2pEvent")]
pub struct Idp2pBehaviour {
    pub gossipsub: Gossipsub,
    pub pair_reqres: RequestResponse<FromPairCodec>,
    pub gossip_reqres: RequestResponse<FromGossipCodec>,
    pub mdns: Mdns,
}

#[derive(Debug)]
pub enum Idp2pEvent {
    Gossipsub(GossipsubEvent),
    Mdns(MdnsEvent),
}

impl From<GossipsubEvent> for Idp2pEvent {
    fn from(event: GossipsubEvent) -> Self {
        Idp2pEvent::Gossipsub(event)
    }
}

impl From<MdnsEvent> for Idp2pEvent {
    fn from(event: MdnsEvent) -> Self {
        Idp2pEvent::Mdns(event)
    }
}

impl Idp2pBehaviour {
    pub fn handle_mdns_event(&mut self, event: MdnsEvent) {
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

pub fn build_gossipsub() -> Gossipsub {
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

pub fn build_swarm(ip: &str, port: u16){
    let local_key = identity::Keypair::generate_ed25519();
    let local_peer_id = PeerId::from(local_key.public());
    println!("Local peer id: {:?}", local_peer_id);
    let transport = libp2p::development_transport(local_key.clone()).await?;
    let mut swarm = {
        let mdns = Mdns::new(Default::default()).await?;

        let behaviour = Idp2pBehaviour {
            gossipsub: build_gossipsub(),
            mdns: mdns,
        };
        let executor = Box::new(|fut| {
            tokio::spawn(fut);
        });
        SwarmBuilder::new(transport, behaviour, local_key.public().to_peer_id())
            .executor(executor)
            .build()
    };
    swarm.listen_on(format!("/ip4/{}/tcp/{}", ip, port).parse()?)?;
}