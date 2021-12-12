use crate::commands::get_command;
use crate::id_swarm::create;
use libp2p::futures::StreamExt;
use libp2p::swarm::SwarmEvent;
use libp2p::Multiaddr;
use std::error::Error;
use structopt::StructOpt;
use tokio::io::{self, AsyncBufReadExt};

#[derive(Debug, StructOpt)]
#[structopt(name = "idp2p", about = "Usage of idp2p.")]
struct Opt {
    #[structopt(short = "p", long = "port", default_value = "0")]
    port: u16,
    #[structopt(short = "d", long = "dial")]
    dial_address: Option<String>,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    env_logger::init();
    let opt = Opt::from_args();
    let mut stdin = io::BufReader::new(io::stdin()).lines();
    let mut swarm = create(opt.port).await?;
    if let Some(to_dial) = opt.dial_address {
        let address: Multiaddr = to_dial.parse().expect("Invalid address.");
        match swarm.dial(address.clone()) {
            Ok(_) => println!("Dialed {:?}", address),
            Err(e) => println!("Dial {:?} failed: {:?}", address, e),
        };
    }
    loop {
        tokio::select! {
            line = stdin.next_line() => {
                let line = line?.expect("stdin closed");
                let cmd = get_command(&line);
                cmd.handle(swarm.behaviour_mut());
            }
            event = swarm.select_next_some() => {
                if let SwarmEvent::NewListenAddr { address, .. } = event {
                    println!("Listening on {:?}", address);
                }
            }
            _ = watcher::async_watch("../target/") =>{}
        }
    }
}

pub mod behaviour;
pub mod commands;
pub mod id_message;
pub mod id_swarm;
pub mod wallet;
pub mod watcher;
