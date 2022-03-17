use crate::behavior::build_swarm;
use crate::behavior::run_command;
use crate::behavior::IdentityRelayEvent;
use dotenv::dotenv;
use idp2p_common::anyhow::Result;
use idp2p_common::ed_secret::EdSecret;
use libp2p::futures::StreamExt;
use libp2p::identity::ed25519::SecretKey;
use libp2p::identity::Keypair;
use libp2p::swarm::SwarmEvent;
use libp2p::gossipsub::IdentTopic;
use tokio::io::AsyncBufReadExt;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv().ok();
    let mut stdin = tokio::io::BufReader::new(tokio::io::stdin()).lines();
    let secret = EdSecret::new();
    let secret_key = SecretKey::from_bytes(secret.to_bytes())?;
    let local_key = Keypair::Ed25519(secret_key.into());
    let mut swarm = build_swarm(local_key.clone()).await;
    let owner = local_key.public().to_peer_id().to_base58();
    let topic = IdentTopic::new(&owner);
    swarm.behaviour_mut().gossipsub.subscribe(&topic).unwrap();
    swarm.listen_on("/ip4/0.0.0.0/tcp/43727".parse()?)?;

    loop {
        tokio::select! {
            line = stdin.next_line() => {
                let line = line?.expect("stdin closed");
                run_command(&line, &mut swarm)?;
            }
            swarm_event = swarm.select_next_some() => {
                match swarm_event {
                    SwarmEvent::NewListenAddr { address, .. } => {
                        println!("Listening on {:?}", address);
                    }
                    SwarmEvent::Behaviour(IdentityRelayEvent::Gossipsub(event)) =>{
                        println!("Got message: {:?}", event);
                        swarm.behaviour_mut().handle_gossip_event(&owner, event).await;
                    }
                    other => { println!("{:?}", other); }
                }
            }
        }
    }
}

pub mod behavior;
