use std::collections::HashMap;
use idp2p_node::store::IdStoreOptions;
use idp2p_common::ed_secret::EdSecret;
use idp2p_core::did::Identity;
use idp2p_node::message::{IdentityMessage, IdentityMessagePayload};
use idp2p_node::node::build_gossipsub;
use idp2p_node::node::build_transport;
use idp2p_node::store::IdStore;
use idp2p_node::IdentityEvent;
use libp2p::{
    futures::StreamExt,
    gossipsub::{Gossipsub, GossipsubEvent, IdentTopic},
    identity::{ed25519::SecretKey, Keypair},
    swarm::{SwarmBuilder, SwarmEvent},
    NetworkBehaviour,
};

use tokio::io::AsyncBufReadExt;
use tokio::sync::mpsc::channel;


#[derive(NetworkBehaviour)]
#[behaviour(out_event = "IdentityNodeEvent")]
pub struct IdentityNodeBehaviour {
    pub gossipsub: Gossipsub,
    #[behaviour(ignore)]
    pub id_store: IdStore,
}


#[derive(Debug)]
pub enum IdentityNodeEvent {
    Gossipsub(GossipsubEvent),
}

impl From<GossipsubEvent> for IdentityNodeEvent {
    fn from(event: GossipsubEvent) -> Self {
        IdentityNodeEvent::Gossipsub(event)
    }
}


fn run_command(input: &str, behaviour: &mut IdentityNodeBehaviour) {
    let split = input.split(" ");
    let input: Vec<&str> = split.collect();
    match input[0] {
        "get" => {
            let topic = IdentTopic::new(input[1]);
            behaviour.gossipsub.subscribe(&topic).unwrap();
            let message = IdentityMessage::new(IdentityMessagePayload::Get);
            let data = idp2p_common::serde_json::to_vec(&message).unwrap();
            behaviour.gossipsub.publish(topic, data).unwrap();
        }
        _ => {}
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut stdin = tokio::io::BufReader::new(tokio::io::stdin()).lines();
    let (tx, mut rx) = channel::<IdentityEvent>(100);
    let secret = EdSecret::new();
    let did = Identity::from_secret(secret.clone());
    std::env::set_var("secret", idp2p_common::encode(&secret.to_bytes()));
    println!("Created: {}", did.id);
    let secret_key = SecretKey::from_bytes(secret.to_bytes())?;
    let local_key = Keypair::Ed25519(secret_key.into());
    let transport = build_transport(local_key.clone()).await;
    let mut swarm = {
        let options = IdStoreOptions{
             owner: did.clone(),
             event_sender: tx.clone(),
             entries: HashMap::new()
        };
        let id_store = IdStore::new(options);
        let behaviour = IdentityNodeBehaviour {
            gossipsub: build_gossipsub(),
            id_store: id_store,
        };
        let executor = Box::new(|fut| {
            tokio::spawn(fut);
        });
        SwarmBuilder::new(transport, behaviour, local_key.public().to_peer_id())
            .executor(executor)
            .build()
    };
    swarm.listen_on(format!("/ip4/0.0.0.0/tcp/43727").parse()?)?;
    let topic = IdentTopic::new(&did.id);
    swarm.behaviour_mut().gossipsub.subscribe(&topic)?;
    swarm.behaviour_mut().id_store.push_did(did);
    loop {
        tokio::select! {
            line = stdin.next_line() => {
                let line = line?.expect("stdin closed");
                run_command(&line, swarm.behaviour_mut());
            }
            swarm_event = swarm.select_next_some() => {
                match swarm_event {
                    SwarmEvent::NewListenAddr { address, .. } => {
                        println!("Listening on {:?}", address);
                    }
                    SwarmEvent::Behaviour(IdentityNodeEvent::Gossipsub(event)) =>{
                        println!("Got message: {:?}", event);
                    }
                    other => {println!("{:?}", other);}
                }
            }
            event = rx.recv() => {
                if let Some(event) = event{
                    match event{
                        _ => {  }
                    }
                }
            }
        }
    }
}
