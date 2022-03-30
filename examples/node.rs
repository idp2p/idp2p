use idp2p_common::anyhow::Result;
use idp2p_common::ed_secret::EdSecret;
use idp2p_core::IdentityEvent;
use idp2p_node::{
    behaviour::IdentityNodeEvent,
    swarm::{build_swarm, NodeOptions},
};
use libp2p::futures::StreamExt;
use libp2p::swarm::SwarmEvent;
use structopt::StructOpt;
use tokio::sync::mpsc::channel;

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
    let (tx, mut rx) = channel::<IdentityEvent>(100);
    let node_options = NodeOptions::new(secret, tx.clone(), opt.connect);
    let mut swarm = build_swarm(node_options).await?;
    loop {
        tokio::select! {
            swarm_event = swarm.select_next_some() => {
                match swarm_event {
                    SwarmEvent::NewListenAddr { address, .. } => {
                        println!("Listening on {:?}", address);
                    }
                    SwarmEvent::Behaviour(IdentityNodeEvent::Gossipsub(event)) =>{
                        swarm.behaviour_mut().handle_gossip_event(event).await;
                    }
                    _ => {  }
                }
            }
            _ = rx.recv() => {

            }
        }
    }
}
