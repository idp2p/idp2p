use anyhow::Result;
use libp2p::Swarm;
use libp2p::{
    gossipsub::{
        Gossipsub, GossipsubConfigBuilder, GossipsubEvent, GossipsubMessage, IdentTopic,
        MessageAuthenticity, MessageId, ValidationMode,
    },
    identity,
    mdns::{Mdns, MdnsEvent},
    swarm::{NetworkBehaviourEventProcess, SwarmBuilder},
    NetworkBehaviour, PeerId,
};
use std::collections::hash_map::DefaultHasher;
use std::collections::HashMap;
use std::collections::HashMap;
use std::error::Error;
use std::hash::{Hash, Hasher};
use std::time::Duration;

#[derive(NetworkBehaviour)]
#[behaviour(event_process = true)]
pub struct IdentityGossipBehaviour {
    pub gossipsub: Gossipsub,
    pub mdns: Mdns,
    #[behaviour(ignore)]
    pub identities: HashMap<String, String>,
}

impl NetworkBehaviourEventProcess<GossipsubEvent> for IdentityGossipBehaviour {
    fn inject_event(&mut self, message: GossipsubEvent) {
        if let GossipsubEvent::Message {
            propagation_source: _,
            message_id: _,
            message,
        } = message
        {
            let id = message.topic.to_string();
            let id_event: IdentityEvent = serde_json::from_slice(&message.data).unwrap();
            match id_event.payload {

            }
        }
    }
}

impl NetworkBehaviourEventProcess<MdnsEvent> for IdentityGossipBehaviour {
    fn inject_event(&mut self, event: MdnsEvent) {
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

pub async fn create() -> Result<Swarm<IdentityGossipBehaviour>, Box<dyn Error>> {
    let local_key = identity::Keypair::generate_ed25519();
    let local_peer_id = PeerId::from(local_key.public());
    println!("Local peer id: {:?}", local_peer_id);

    let transport = libp2p::development_transport(local_key.clone()).await?;

    let mut swarm = {
        let message_id_fn = |message: &GossipsubMessage| {
            let mut s = DefaultHasher::new();
            message.data.hash(&mut s);
            MessageId::from(s.finish().to_string())
        };

        let gossipsub_config = GossipsubConfigBuilder::default()
            .heartbeat_interval(Duration::from_secs(10))
            .validation_mode(ValidationMode::Strict)
            .message_id_fn(message_id_fn)
            .build()
            .expect("Valid config");

        let gossipsub_result = libp2p::gossipsub::Gossipsub::new(
            MessageAuthenticity::Signed(local_key),
            gossipsub_config,
        );
        let gossipsub = gossipsub_result.expect("Correct configuration");
        let mdns = libp2p::mdns::Mdns::new(Default::default()).await?;
        let behaviour = IdentityGossipBehaviour {
            gossipsub: gossipsub,
            mdns: mdns,
            identities: HashMap::new(),
        };
        SwarmBuilder::new(transport, behaviour, local_peer_id)
            .executor(Box::new(|fut| {
                tokio::spawn(fut);
            }))
            .build()
    };
    swarm.listen_on("/ip4/0.0.0.0/tcp/43727")?;
    Ok(swarm)
}
