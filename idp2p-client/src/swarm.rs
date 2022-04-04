use crate::IdConfig;
use crate::behaviour::IdentityClientBehaviour;
use crate::builder::build_gossipsub;
use crate::builder::build_transport;
use idp2p_common::anyhow::Result;
use idp2p_core::IdentityEvent;
use idp2p_core::store::IdStore;
use libp2p::identity::ed25519::SecretKey;
use libp2p::identity::Keypair;
use libp2p::mdns::Mdns;
use libp2p::swarm::SwarmBuilder;
use libp2p::Swarm;
use std::sync::Arc;
use tokio::sync::mpsc::Sender;

pub struct IdSwarmOptions {
    config: IdConfig,
    event_sender: Sender<IdentityEvent>,
}

impl IdSwarmOptions{
    pub fn new(config: IdConfig, tx: Sender<IdentityEvent>) -> IdSwarmOptions{
        IdSwarmOptions{
            config: config,
            event_sender: tx 
        }
    }
}

pub async fn build_swarm(options: IdSwarmOptions) -> Result<Swarm<IdentityClientBehaviour>> {
    let secret_key = SecretKey::from_bytes(&mut options.config.secret.clone())?;
    let local_key = Keypair::Ed25519(secret_key.into());
    let transport = build_transport(local_key.clone()).await;
    let id_store = IdStore::new(options.config.identities, options.event_sender);
    let id_store = Arc::new(id_store);
    let mut swarm = {
        let behaviour = IdentityClientBehaviour {
            gossipsub: build_gossipsub(),
            mdns: Mdns::new(Default::default()).await?,
            id_store: id_store,
        };
        let executor = Box::new(|fut| {
            tokio::spawn(fut);
        });
        SwarmBuilder::new(transport, behaviour, local_key.public().to_peer_id())
            .executor(executor)
            .build()
    };
    swarm.listen_on(format!("/ip4/0.0.0.0/tcp/{}", options.config.listen_port).parse()?)?;
    Ok(swarm)
}
