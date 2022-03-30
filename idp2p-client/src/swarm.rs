use libp2p::PeerId;
use libp2p::Multiaddr;
use crate::behaviour::IdentityClientBehaviour;
use crate::builder::build_gossipsub;
use crate::builder::build_transport;
use crate::persiter::FilePersister;
use idp2p_common::anyhow::Result;
use idp2p_common::ed_secret::EdSecret;
use idp2p_core::store::IdEntry;
use idp2p_core::store::IdStore;
use idp2p_wallet::store::WalletStore;
use libp2p::identity::ed25519::SecretKey;
use libp2p::identity::Keypair;
use libp2p::mdns::Mdns;
use libp2p::swarm::SwarmBuilder;
use libp2p::Swarm;
use std::collections::HashMap;
use std::str::FromStr;
use std::sync::Arc;

pub struct ClientOptions {
    port: u16,
    secret: EdSecret,
    connect: String,
    identities: HashMap<String, IdEntry>,
}

pub async fn build_swarm(options: ClientOptions) -> Result<Swarm<IdentityClientBehaviour>> {
    let secret_key = SecretKey::from_bytes(options.secret.to_bytes())?;
    let local_key = Keypair::Ed25519(secret_key.into());
    let transport = build_transport(local_key.clone()).await;
    let id_store = Arc::new(IdStore::new(options.identities));
   
    //tokio::spawn(purge_id_events(id_store.clone(), options.id_event_sender));

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
    swarm.listen_on(format!("/ip4/0.0.0.0/tcp/{}", options.port).parse()?)?;
    let split: Vec<&str> = options.connect.split("/").collect();
    let to_dial = format!("/ip4/{}/tcp/{}", split[2], split[4]);
    let addr: Multiaddr = to_dial.parse().unwrap();
    let peer_id = PeerId::from_str(split[6])?;
    swarm.dial(addr)?;
    swarm.behaviour_mut().gossipsub.add_explicit_peer(&peer_id);
    Ok(swarm)
}
