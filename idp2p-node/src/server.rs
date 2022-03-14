use crate::build_gossipsub;
use crate::build_transport;
use crate::message::{IdentityMessage, IdentityMessagePayload};
use crate::store::IdStore;
use crate::IdentityEvent;
use idp2p_common::{anyhow::Result, serde_json};
use idp2p_core::did::Identity;
use libp2p::identity::Keypair;
use libp2p::relay::v2::relay::{self, Relay};
use libp2p::{
    gossipsub::{Gossipsub, GossipsubEvent},
    identify::{Identify, IdentifyConfig, IdentifyEvent},
    rendezvous,
    swarm::{NetworkBehaviourEventProcess, Swarm, SwarmBuilder},
    NetworkBehaviour,
};
use tokio::sync::mpsc::Sender;

pub struct SwarmOptions {
    pub port: u16,
    pub owner: Identity,
    pub tx: Sender<IdentityEvent>,
}

#[derive(NetworkBehaviour)]
#[behaviour(event_process = true)]
pub struct IdentityServerBehaviour {
    pub identify: Identify,
    pub gossipsub: Gossipsub,
    #[behaviour(ignore)]
    pub id_store: IdStore,
    pub relay: Relay,
    pub rendezvous: rendezvous::server::Behaviour,
}

impl NetworkBehaviourEventProcess<GossipsubEvent> for IdentityServerBehaviour {
    fn inject_event(&mut self, message: GossipsubEvent) {
        println!("Got message: {:?}", message);
        if let GossipsubEvent::Message {
            propagation_source: _,
            message_id: _,
            message,
        } = message
        {
            let topic = message.topic.to_string();
            let err_msg = "Message is not well-formed. It should be json";
            let message: IdentityMessage = serde_json::from_slice(&message.data).expect(err_msg);
            match &message.payload {
                IdentityMessagePayload::Get => {
                    self.id_store.handle_get(&topic);
                }
                IdentityMessagePayload::Post { digest, identity } => {
                    let result = self.id_store.handle_post(digest, identity);
                    match result {
                        Ok(_) => {}
                        Err(_) => {}
                    }
                }
                _ => {}
            }
        }
    }
}

impl NetworkBehaviourEventProcess<IdentifyEvent> for IdentityServerBehaviour {
    fn inject_event(&mut self, event: IdentifyEvent) {
        if let IdentifyEvent::Received { peer_id, .. } = event {
            println!("Identify event: {}", peer_id);
        }
    }
}

impl NetworkBehaviourEventProcess<relay::Event> for IdentityServerBehaviour {
    fn inject_event(&mut self, event: relay::Event) {
        println!("{:?}", event);
    }
}

impl NetworkBehaviourEventProcess<rendezvous::server::Event> for IdentityServerBehaviour {
    fn inject_event(&mut self, e: rendezvous::server::Event) {
        println!("{:?}", e);
    }
}

pub async fn create_server_swarm(options: SwarmOptions) -> Result<Swarm<IdentityServerBehaviour>> {
    let local_key = Keypair::generate_ed25519();
    let transport = build_transport(local_key.clone()).await;
    let mut swarm = {
        let identify = Identify::new(IdentifyConfig::new(
            "rendezvous-example/1.0.0".to_string(),
            local_key.public(),
        ));
        let rendezvous = rendezvous::server::Behaviour::new(rendezvous::server::Config::default());
        let relay = Relay::new(local_key.public().to_peer_id(), Default::default());
        let id_store = IdStore::new(options.tx.clone(), options.owner);
        let behaviour = IdentityServerBehaviour {
            identify: identify,
            rendezvous: rendezvous,
            gossipsub: build_gossipsub(),
            relay: relay,
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
