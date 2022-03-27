use idp2p_node::{
    get_peer_info,
    behaviour::{build_swarm, IdentityRelayEvent}
};
use idp2p_common::anyhow::Result;
use idp2p_common::ed_secret::EdSecret;
use libp2p::futures::StreamExt;
use libp2p::identity::ed25519::SecretKey;
use libp2p::identity::Keypair;
use libp2p::swarm::SwarmEvent;
use libp2p::Multiaddr;
use libp2p::PeerId;
use std::str::FromStr;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
#[structopt(name = "idp2p", about = "Usage of idp2p.")]
struct Opt {
    #[structopt(short = "c", long = "connect")]
    connect: Option<String>,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let opt = Opt::from_args();
    let secret = EdSecret::new();
    let secret_key = SecretKey::from_bytes(secret.to_bytes())?;
    let local_key = Keypair::Ed25519(secret_key.into());
    let mut swarm = build_swarm(local_key.clone()).await;
    swarm.listen_on("/ip4/0.0.0.0/tcp/43727".parse()?)?;
    if let Some(connect) = opt.connect {
        let (addr, peer_id) = get_peer_info(&connect)?;
        swarm.dial(addr)?;
        swarm.behaviour_mut().gossipsub.add_explicit_peer(&peer_id);
    }
    loop {
        tokio::select! {
            swarm_event = swarm.select_next_some() => {
                match swarm_event {
                    SwarmEvent::NewListenAddr { address, .. } => {
                        println!("Listening on {:?}", address);
                    }
                    SwarmEvent::Behaviour(IdentityRelayEvent::Gossipsub(event)) =>{
                        swarm.behaviour_mut().handle_gossip_event(event).await;
                    }
                    other => { println!("{:?}", other); }
                }
            }
        }
    }
}
