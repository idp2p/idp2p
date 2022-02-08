use crate::behaviour::IdentityGossipEvent;
use crate::behaviour::IdentityGossipBehaviour;
use anyhow::Result;
use libp2p::Swarm;
use libp2p::{
    gossipsub::{
        GossipsubConfigBuilder, GossipsubMessage,
        MessageAuthenticity, MessageId, ValidationMode,
    },
    identity,
    swarm::{ SwarmBuilder}, PeerId,
};
use std::collections::hash_map::DefaultHasher;
use std::collections::HashMap;
use std::error::Error;
use std::hash::{Hash, Hasher};
use std::time::Duration;

pub async fn create_swarm(
    port: u16,
    sender: tokio::sync::mpsc::Sender<IdentityGossipEvent>,
) -> Result<Swarm<IdentityGossipBehaviour>, Box<dyn Error>> {
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
            .validation_mode(ValidationMode::Anonymous)
            .message_id_fn(message_id_fn)
            .build()
            .expect("Valid config");

        let gossipsub_result =
            libp2p::gossipsub::Gossipsub::new(MessageAuthenticity::Anonymous, gossipsub_config);
        let gossipsub = gossipsub_result.expect("Correct configuration");
        let mdns = libp2p::mdns::Mdns::new(Default::default()).await?;
        let behaviour = IdentityGossipBehaviour {
            gossipsub: gossipsub,
            mdns: mdns,
            identities: HashMap::new(),
            sender: sender,
        };
        SwarmBuilder::new(transport, behaviour, local_peer_id)
            .executor(Box::new(|fut| {
                tokio::spawn(fut);
            }))
            .build()
    };
    swarm.listen_on(format!("/ip4/0.0.0.0/tcp/{}", port).parse()?)?;
    //swarm.listen_on("/ip4/0.0.0.0/tcp/43727".parse()?)?;
    Ok(swarm)
}
