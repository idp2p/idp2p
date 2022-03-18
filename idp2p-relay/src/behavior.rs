use idp2p_node::node::build_gossipsub;
use idp2p_node::node::build_transport;
use idp2p_node::req_res::IdRequest;
use idp2p_node::req_res::IdResponse;
use libp2p::identity::Keypair;
use libp2p::swarm::SwarmBuilder;
use libp2p::Multiaddr;
use libp2p::PeerId;
use libp2p::{
    gossipsub::{Gossipsub, GossipsubEvent, IdentTopic},
    request_response::RequestResponseEvent,
    swarm::Swarm,
    NetworkBehaviour,
};
use std::str::FromStr;

pub async fn build_swarm(local_key: Keypair) -> Swarm<IdentityRelayBehaviour> {
    let transport = build_transport(local_key.clone()).await;
    let swarm = {
        let behaviour = IdentityRelayBehaviour {
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
    pub gossipsub: Gossipsub,
}

#[derive(Debug)]
pub enum IdentityRelayEvent {
    Gossipsub(GossipsubEvent),
    RequestResponse(RequestResponseEvent<IdRequest, IdResponse>),
}

impl From<GossipsubEvent> for IdentityRelayEvent {
    fn from(event: GossipsubEvent) -> Self {
        IdentityRelayEvent::Gossipsub(event)
    }
}

impl From<RequestResponseEvent<IdRequest, IdResponse>> for IdentityRelayEvent {
    fn from(event: RequestResponseEvent<IdRequest, IdResponse>) -> Self {
        IdentityRelayEvent::RequestResponse(event)
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
        }
        "subscribe" => {
            swarm
                .behaviour_mut()
                .gossipsub
                .subscribe(&IdentTopic::new(input[1]))?;
        }
        _ => {}
    }
    Ok(())
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
}
