use idp2p_node::node::build_gossipsub;
use idp2p_node::node::build_transport;
use libp2p::identity::Keypair;
use libp2p::swarm::SwarmBuilder;
use libp2p::PeerId;
use libp2p::{
    gossipsub::{Gossipsub, GossipsubEvent, IdentTopic},
    swarm::Swarm,
    NetworkBehaviour,
};

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
    Gossipsub(GossipsubEvent)
}

impl From<GossipsubEvent> for IdentityRelayEvent {
    fn from(event: GossipsubEvent) -> Self {
        IdentityRelayEvent::Gossipsub(event)
    }
}

impl IdentityRelayBehaviour {
    pub async fn handle_gossip_event(&mut self, event: GossipsubEvent) {
        if let GossipsubEvent::Subscribed {
            peer_id,
            topic
        } = event
        {
            if self.is_authorized_peer(peer_id){
                let new_topic = IdentTopic::new(topic.into_string());
                self.gossipsub.subscribe(&new_topic).unwrap();
            }
        }
    }

    fn is_authorized_peer(&self, peer_id: PeerId) -> bool{
        true
    }
}
