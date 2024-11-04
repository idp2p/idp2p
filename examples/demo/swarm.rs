use libp2p::{
    mdns, noise, ping,gossipsub,
    request_response::{self, ProtocolSupport},
    swarm::{behaviour, NetworkBehaviour},
    tcp, yamux, Multiaddr, StreamProtocol,
};
use serde::{Deserialize, Serialize};
use std::{error::Error, hash::{DefaultHasher, Hash, Hasher}, sync::Arc, time::Duration};

type RequestResponseBehaviour = request_response::cbor::Behaviour<FileRequest, FileResponse>;
type GossipsubBehaviour = libp2p::gossipsub::Behaviour;
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
struct FileRequest(String);
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct FileResponse(Vec<u8>);

#[derive(NetworkBehaviour)]
struct Idp2pBehaviour {
    request_response: RequestResponseBehaviour,
    gossipsub: GossipsubBehaviour,
    mdns: mdns::tokio::Behaviour,
}
pub fn create_swarm(port: u16) -> Result<(), Box<dyn Error>>{
    let mut swarm = libp2p::SwarmBuilder::with_new_identity()
        .with_tokio()
        .with_tcp(
            tcp::Config::default(),
            noise::Config::new,
            yamux::Config::default,
        )?
        .with_quic()
        .with_behaviour(|key| {
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
            let gossipsub = gossipsub::Behaviour::new(
                gossipsub::MessageAuthenticity::Signed(key.clone()),
                gossipsub_config,
            )?;

            let mdns =
                mdns::tokio::Behaviour::new(mdns::Config::default(), key.public().to_peer_id())?;
            let behaviour = Idp2pBehaviour {
                request_response: request_response::cbor::Behaviour::new(
                    [(StreamProtocol::new("/idp2p/1"), ProtocolSupport::Full)],
                    request_response::Config::default(),
                ),
                gossipsub: gossipsub,
                mdns: mdns,
            };
            Ok(behaviour)
        })?
        .with_swarm_config(|c| c.with_idle_connection_timeout(Duration::from_secs(60)))
        .build();

    swarm.listen_on("/ip4/0.0.0.0/tcp/43727".parse().unwrap())?;
    Ok(())
}