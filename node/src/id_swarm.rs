use crate::behaviour::IdentityGossipBehaviour;
use libp2p::gossipsub::{
    GossipsubConfigBuilder, GossipsubMessage, MessageAuthenticity, MessageId, ValidationMode,
};
use libp2p::swarm::SwarmBuilder;
use libp2p::Swarm;
use libp2p::{identity, PeerId};
use std::collections::hash_map::DefaultHasher;
use std::collections::HashMap;
use std::error::Error;
use std::hash::{Hash, Hasher};
use std::time::Duration;

pub async fn create(port: u16) -> Result<Swarm<IdentityGossipBehaviour>, Box<dyn Error>> {
    // Create a random PeerId
    let local_key = identity::Keypair::generate_ed25519();
    let local_peer_id = PeerId::from(local_key.public());
    println!("Local peer id: {:?}", local_peer_id);

    let transport = libp2p::development_transport(local_key.clone()).await?;

    // Create a Swarm to manage peers and events.
    let mut swarm = {
        let message_id_fn = |message: &GossipsubMessage| {
            let mut s = DefaultHasher::new();
            message.data.hash(&mut s);
            MessageId::from(s.finish().to_string())
        };

        let gossipsub_config = GossipsubConfigBuilder::default()
            .heartbeat_interval(Duration::from_secs(10)) // This is set to aid debugging by not cluttering the log space
            .validation_mode(ValidationMode::Strict) // This sets the kind of message validation. The default is Strict (enforce message signing)
            .message_id_fn(message_id_fn) // content-address messages. No two messages of the
            .build()
            .expect("Valid config");

        let gossipsub_reuslt = libp2p::gossipsub::Gossipsub::new(
            MessageAuthenticity::Signed(local_key),
            gossipsub_config,
        );
        let gossipsub = gossipsub_reuslt.expect("Correct configuration");
        let mdns = libp2p::mdns::Mdns::new(Default::default()).await?;
        let behaviour = IdentityGossipBehaviour {
            gossipsub: gossipsub,
            mdns: mdns,
            identities: HashMap::new()
        };
        SwarmBuilder::new(transport, behaviour, local_peer_id)
            .executor(Box::new(|fut| {
                tokio::spawn(fut);
            }))
            .build()
    };
    // Listen on all interfaces and whatever port the OS assigns
    swarm.listen_on(format!("/ip4/0.0.0.0/tcp/{}", port).parse()?)?;
    Ok(swarm)
}
