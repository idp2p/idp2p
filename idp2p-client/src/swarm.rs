use idp2p_common::anyhow::Result;
use idp2p_core::store::IdEntry;
use idp2p_core::store::IdStore;
use idp2p_core::IdentityEvent;
use libp2p::identity::ed25519::SecretKey;
use libp2p::identity::Keypair;
use libp2p::swarm::SwarmBuilder;
use libp2p::Swarm;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::mpsc::Sender;

use libp2p::{
    gossipsub::{Gossipsub, GossipsubEvent},
    mdns::{Mdns, MdnsEvent},
    NetworkBehaviour,
};

use crate::builder::build_gossipsub;
use crate::builder::build_transport;

#[derive(NetworkBehaviour)]
#[behaviour(out_event = "IdentityClientEvent")]
pub struct IdentityClientBehaviour {
    pub mdns: Mdns,
    pub gossipsub: Gossipsub,
    #[behaviour(ignore)]
    pub id_store: Arc<IdStore>,
}

impl IdentityClientBehaviour{
    async fn handle_event(&mut self, event: GossipsubEvent) {
        if let GossipsubEvent::Message {
            propagation_source: _,
            message_id: _,
            message,
        } = event
        {
            let topic = message.topic.to_string();
            if idp2p_common::is_idp2p(&topic) {
                let message = idp2p_core::message::IdentityMessage::from_bytes(&message.data);
                match &message.payload {
                    idp2p_core::message::IdentityMessagePayload::Get => {
                        self.id_store.handle_get(&topic).await;
                    }
                    idp2p_core::message::IdentityMessagePayload::Post { digest, identity } => {
                        //self.id_store.handle_post(digest, identity).await;
                    }
                    _ => {}
                }
            }
        }
    }
}

#[derive(Debug)]
pub enum IdentityClientEvent {
    Mdns(MdnsEvent),
    Gossipsub(GossipsubEvent),
}

impl From<MdnsEvent> for IdentityClientEvent {
    fn from(event: MdnsEvent) -> Self {
        IdentityClientEvent::Mdns(event)
    }
}
impl From<GossipsubEvent> for IdentityClientEvent {
    fn from(event: GossipsubEvent) -> Self {
        IdentityClientEvent::Gossipsub(event)
    }
}
pub struct IdSwarmOptions {
    port: u16,
    secret: Vec<u8>,
    identities: HashMap<String, IdEntry>,
    event_sender: Sender<IdentityEvent>,
}

pub async fn build_swarm(options: IdSwarmOptions) -> Result<Swarm<IdentityClientBehaviour>> {
    let secret_key = SecretKey::from_bytes(&mut options.secret.clone())?;
    let local_key = Keypair::Ed25519(secret_key.into());
    let transport = build_transport(local_key.clone()).await;
    let id_store = IdStore::new(options.identities, options.event_sender);
    let id_store = Arc::new(id_store);
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
    Ok(swarm)
}
