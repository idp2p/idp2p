use idp2p_node::node::build_gossipsub;
use idp2p_node::node::build_transport;
use libp2p::identity::Keypair;
use libp2p::swarm::SwarmBuilder;
use libp2p::Multiaddr;
use libp2p::PeerId;
use libp2p::{
    gossipsub::{Gossipsub, GossipsubEvent, IdentTopic},
    mdns::{Mdns, MdnsEvent},
    swarm::Swarm,
    NetworkBehaviour,
};
use std::str::FromStr;

pub async fn build_swarm(local_key: Keypair) -> Swarm<IdentityRelayBehaviour> {
    let transport = build_transport(local_key.clone()).await;
    let swarm = {
        let behaviour = IdentityRelayBehaviour {
            mdns: Mdns::new(Default::default()).await.unwrap(),
            gossipsub: build_gossipsub(),
        };
        let executor = Box::new(|fut| {
            tokio::spawn(fut);
        });
        SwarmBuilder::new(transport, behaviour, local_key.public().to_peer_id())
            .executor(executor)
            .build()
    };
    return swarm;
}

#[derive(NetworkBehaviour)]
#[behaviour(out_event = "IdentityRelayEvent")]
pub struct IdentityRelayBehaviour {
    mdns: Mdns,
    pub gossipsub: Gossipsub,
}

impl IdentityRelayBehaviour {
    pub async fn handle_gossip_event(&mut self, owner: &str, event: GossipsubEvent) {
        if let GossipsubEvent::Message {
            propagation_source: _,
            message_id: _,
            message,
        } = event
        {
            let topic = message.topic.to_string();
            if topic == owner {
                let new_topic = IdentTopic::new(idp2p_common::encode(&message.data));
                self.gossipsub.subscribe(&new_topic).unwrap();
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
}

#[derive(Debug)]
pub enum IdentityRelayEvent {
    Mdns(MdnsEvent),
    Gossipsub(GossipsubEvent),
}

impl From<GossipsubEvent> for IdentityRelayEvent {
    fn from(event: GossipsubEvent) -> Self {
        IdentityRelayEvent::Gossipsub(event)
    }
}

impl From<MdnsEvent> for IdentityRelayEvent {
    fn from(event: MdnsEvent) -> Self {
        IdentityRelayEvent::Mdns(event)
    }
}

pub fn run_command(
    input: &str,
    swarm: &mut Swarm<IdentityRelayBehaviour>,
) -> idp2p_common::anyhow::Result<()> {
    let split = input.split(" ");
    let input: Vec<&str> = split.collect();
    match input[0] {
        "connect" => {
            let to_dial = format!("/ip4/{}/tcp/{}", input[1], input[2]);
            let addr: Multiaddr = to_dial.parse().unwrap();
            let peer_id = PeerId::from_str(input[3])?;
            swarm.dial(addr)?;
            swarm.behaviour_mut().gossipsub.add_explicit_peer(&peer_id);
            swarm.behaviour_mut().gossipsub.subscribe(&IdentTopic::new(input[3]))?;
        }
        "subscribe" => {
            let topic = IdentTopic::new(input[1]);
            swarm
                .behaviour_mut()
                .gossipsub
                .publish(topic, input[2].as_bytes())
                .unwrap();
        }
        _ => {}
    }
    Ok(())
}
