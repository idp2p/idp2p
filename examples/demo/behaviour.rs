use idp2p_p2p::behaviour::{create_gossipsub, create_reqres, IdRequestResponseBehaviour};
use libp2p::{
    gossipsub::Behaviour as GossipsubBehaviour, mdns, noise,
    swarm::NetworkBehaviour,
    tcp, yamux,  Swarm,
};
use std::{error::Error, time::Duration};

#[derive(NetworkBehaviour)]
pub(crate) struct Idp2pBehaviour {
    pub(crate) request_response: IdRequestResponseBehaviour,
    pub(crate) gossipsub: GossipsubBehaviour,
    pub(crate) mdns: mdns::tokio::Behaviour,
}

pub struct IdMessageHandler<S: KvStore> {
    engine: Engine,
    kv_store: Arc<S>,
    swarm: Swarm<Idp2pBehaviour>,
    components: HashMap<String, Module>,
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
