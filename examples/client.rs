/*use core::libp2p::{futures::StreamExt, swarm::SwarmEvent};
use gossip::{
    behaviour::IdentityGossipEvent,
    swarm::{build_gossip_swarm, GossipOptions},
};
use structopt::StructOpt;
use tokio::io::AsyncBufReadExt;

#[derive(Debug, StructOpt)]
#[structopt(name = "idp2pgossip", about = "Usage of idp2p gossip.")]
struct Opt {
    #[structopt(
        short = "l",
        long = "listen",
        default_value = "/ip4/127.0.0.1/tcp/43727"
    )]
    listen: String,
    #[structopt(short = "d", long = "dial")]
    dial: Option<String>,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let opt = Opt::from_args();
    let options = GossipOptions {
        to_dial: opt.dial,
        listen: opt.listen,
    };
    let mut stdin = tokio::io::BufReader::new(tokio::io::stdin()).lines();
    let mut swarm = build_gossip_swarm(options).await?;
    loop {
        tokio::select! {
            line = stdin.next_line() => {
                let line = line?.expect("stdin closed");
                println!("{line}");
            }
            swarm_event = swarm.select_next_some() => {
                match swarm_event {
                    SwarmEvent::NewListenAddr { address, .. } => {
                       println!("Listening on {:?}", address);
                    }
                    SwarmEvent::Behaviour(IdentityGossipEvent::Gossipsub(event)) => {
                        swarm.behaviour_mut().handle_req_event(event)?;
                    }
                    SwarmEvent::Behaviour(IdentityGossipEvent::RequestResponse(event)) => {
                        swarm.behaviour_mut().handle_req_event(event)?;
                    }
                    _ => {  }
                }
            }
        }
    }
}*/

fn main(){}
