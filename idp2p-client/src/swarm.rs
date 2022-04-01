use libp2p::PeerId;
use libp2p::Multiaddr;
use crate::behaviour::IdentityClientBehaviour;
use crate::builder::build_gossipsub;
use crate::builder::build_transport;
use idp2p_common::anyhow::Result;
use idp2p_common::ed_secret::EdSecret;
use idp2p_core::store::IdEntry;
use idp2p_core::store::IdStore;
use libp2p::identity::ed25519::SecretKey;
use libp2p::identity::Keypair;
use libp2p::mdns::Mdns;
use libp2p::swarm::SwarmBuilder;
use libp2p::Swarm;
use std::collections::HashMap;
use std::str::FromStr;
use std::sync::Arc;

pub struct ClientOptions {
    secret: EdSecret,
    identities: HashMap<String, IdEntry>,
}

pub async fn build_swarm(options: ClientOptions) -> Result<Swarm<IdentityClientBehaviour>> {
    let secret_key = SecretKey::from_bytes(options.secret.to_bytes())?;
    let local_key = Keypair::Ed25519(secret_key.into());
    let transport = build_transport(local_key.clone()).await;
    let id_store = Arc::new(IdStore::new(options.identities));
   
    //tokio::spawn(purge_id_events(id_store.clone(), options.id_event_sender));

    let swarm = {
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
    Ok(swarm)
}
