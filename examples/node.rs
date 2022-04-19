use idp2p_common::anyhow::Result;
use idp2p_node::swarm::{NodeOptions, build_swarm, IdentityNodeEvent};
use libp2p::futures::StreamExt;
use libp2p::swarm::SwarmEvent;
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
    let node_options = NodeOptions::new( opt.connect);
    let mut swarm = build_swarm(node_options).await?;
    loop {
        tokio::select! {
            swarm_event = swarm.select_next_some() => {
                match swarm_event {
                    SwarmEvent::NewListenAddr { address, .. } => {
                        println!("Listening on {:?}", address);
                    }
                    SwarmEvent::Behaviour(IdentityNodeEvent::Gossipsub(event)) =>{
                        swarm.behaviour_mut().handle_gossip_event(event).await?;
                    }
                    _ => {  }
                }
            }
        }
    }
}
