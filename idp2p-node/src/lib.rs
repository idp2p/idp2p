use crate::{
    behaviour::IdentityNodeBehaviour,
    builder::{build_gossipsub, build_transport},
    store::{Client, ClientStore},
};
use idp2p_common::anyhow::Result;
use idp2p_common::ed_secret::EdSecret;
use idp2p_core::store::IdEntry;
use idp2p_core::store::IdStore;
use idp2p_core::IdentityEvent;
use libp2p::{
    identity::{ed25519::SecretKey, Keypair},
    mdns::Mdns,
    swarm::SwarmBuilder,
    Multiaddr, PeerId, Swarm,
};
use std::collections::HashMap;
use std::collections::HashSet;
use std::str::FromStr;
use std::sync::Arc;
use tokio::sync::mpsc::Sender;

pub struct NodeOptions {
    port: u16,
    secret: EdSecret,
    connect: Option<String>,
    id_event_sender: Sender<IdentityEvent>,
    clients: HashMap<String, Client>,
    identities: HashMap<String, IdEntry>,
    subscriptions: HashSet<String>,
}

impl NodeOptions {
    pub fn new(
        secret: EdSecret,
        id_event_sender: Sender<IdentityEvent>,
        connect: Option<String>,
    ) -> Self {
        NodeOptions {
            port: 43727,
            secret: secret,
            connect: connect,
            clients: HashMap::new(),
            identities: HashMap::new(),
            subscriptions: HashSet::new(),
            id_event_sender: id_event_sender,
        }
    }
}

pub async fn build_swarm(options: NodeOptions) -> Result<Swarm<IdentityNodeBehaviour>> {
    let secret_key = SecretKey::from_bytes(options.secret.to_bytes())?;
    let local_key = Keypair::Ed25519(secret_key.into());
    let transport = build_transport(local_key.clone()).await;
    let id_store = Arc::new(IdStore::new(options.identities));
    tokio::spawn(purge_id_events(id_store.clone(), options.id_event_sender));
    let client_store = Arc::new(ClientStore::new(options.clients, options.subscriptions));
    let mut swarm = {
        let behaviour = IdentityNodeBehaviour {
            gossipsub: build_gossipsub(),
            mdns: Mdns::new(Default::default()).await?,
            identities: id_store,
            clients: client_store,
        };
        let executor = Box::new(|fut| {
            tokio::spawn(fut);
        });
        SwarmBuilder::new(transport, behaviour, local_key.public().to_peer_id())
            .executor(executor)
            .build()
    };
    swarm.listen_on(format!("/ip4/0.0.0.0/tcp/{}", options.port).parse()?)?;
    if let Some(connect) = options.connect {
        let split: Vec<&str> = connect.split("/").collect();
        let to_dial = format!("/ip4/{}/tcp/{}", split[2], split[4]);
        let addr: Multiaddr = to_dial.parse().unwrap();
        let peer_id = PeerId::from_str(split[6])?;
        swarm.dial(addr)?;
        swarm.behaviour_mut().gossipsub.add_explicit_peer(&peer_id);
    }
    Ok(swarm)
}

async fn purge_id_events(id_store: Arc<IdStore>, event_sender: Sender<IdentityEvent>) {}

pub mod behaviour;
pub(crate) mod builder;
pub mod store;
