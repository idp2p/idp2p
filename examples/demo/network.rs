use futures::channel::mpsc;
use libp2p::{
    gossipsub::{self, Behaviour as GossipsubBehaviour},
    identity::Keypair,
    mdns, noise,
    request_response::{cbor::Behaviour as ReqResBehaviour, ProtocolSupport},
    swarm::NetworkBehaviour,
    tcp, yamux, StreamProtocol, Swarm,
};
use std::{
    error::Error, hash::{DefaultHasher, Hash, Hasher}, sync::Arc, time::Duration
};

use crate::store::InMemoryKvStore;

#[derive(NetworkBehaviour)]
pub(crate) struct Idp2pBehaviour {
    pub(crate) request_response: ReqResBehaviour<Vec<u8>, ()>,
    pub(crate) gossipsub: GossipsubBehaviour,
    pub(crate) mdns: mdns::tokio::Behaviour,
}

pub fn create_gossipsub(key: &Keypair) -> anyhow::Result<GossipsubBehaviour> {
    let message_id_fn = |message: &gossipsub::Message| {
        let mut s = DefaultHasher::new();
        message.data.hash(&mut s);
        gossipsub::MessageId::from(s.finish().to_string())
    };

    // Set a custom gossipsub configuration
    let gossipsub_config = gossipsub::ConfigBuilder::default()
        .heartbeat_interval(Duration::from_secs(10)) // This is set to aid debugging by not cluttering the log space
        .validation_mode(gossipsub::ValidationMode::Strict) // This sets the kind of message validation. The default is Strict (enforce message signing)
        .message_id_fn(message_id_fn) // content-address messages. No two messages of the same content will be propagated.
        .build()
        .map_err(|msg| tokio::io::Error::new(tokio::io::ErrorKind::Other, msg))?; // Temporary hack because `build` does not return a proper `std::error::Error`.

    // build a gossipsub network behaviour
    let gossipsub = GossipsubBehaviour::new(
        gossipsub::MessageAuthenticity::Signed(key.clone()),
        gossipsub_config,
    )
    .map_err(anyhow::Error::msg)?;
    Ok(gossipsub)
}

pub fn create_reqres() -> ReqResBehaviour<Vec<u8>, ()> {
    libp2p::request_response::cbor::Behaviour::new(
        [(StreamProtocol::new("/idp2p/1"), ProtocolSupport::Full)],
        libp2p::request_response::Config::default(),
    )
}

pub fn create_swarm(port: u16) -> Result<Swarm<Idp2pBehaviour>, Box<dyn Error>> {
    let mut swarm = libp2p::SwarmBuilder::with_new_identity()
        .with_tokio()
        .with_tcp(
            tcp::Config::default(),
            noise::Config::new,
            yamux::Config::default,
        )?
        .with_quic()
        .with_behaviour(|key| {
            let mdns =
                mdns::tokio::Behaviour::new(mdns::Config::default(), key.public().to_peer_id())?;
            let behaviour = Idp2pBehaviour {
                mdns,
                request_response: create_reqres(),
                gossipsub: create_gossipsub(key)?,
            };
            Ok(behaviour)
        })?
        .with_swarm_config(|c| c.with_idle_connection_timeout(Duration::from_secs(60)))
        .build();

    swarm.listen_on(format!("/ip4/0.0.0.0/tcp/{port}").parse().unwrap())?;
    Ok(swarm)
}

pub(crate) struct IdNetworkEventLoop {
    swarm: Swarm<Idp2pBehaviour>,
    store: Arc<InMemoryKvStore>,
    event_receiver: mpsc::Receiver<IdNetworkEvent>,
    event_sender: mpsc::Sender<IdNetworkEvent>
}

pub(crate) enum IdNetworkEvent {
    
}