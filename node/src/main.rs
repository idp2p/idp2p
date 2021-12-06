use crate::commands::get_command;
use crate::gossipsub_swarm::*;
use libp2p::futures::StreamExt;
use libp2p::swarm::SwarmEvent;
use std::error::Error;
use tokio::io::{self, AsyncBufReadExt};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    env_logger::init();
    let args: Vec<String> = std::env::args().collect();
    let port = (args.len() > 1)
        .then(|| args[1].clone().parse::<u16>().unwrap())
        .unwrap_or(0);

    let mut stdin = io::BufReader::new(io::stdin()).lines();
    let mut swarm = create(port).await?;
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
        }
    }
}

pub mod behaviour;
pub mod commands;
pub mod gossipsub_swarm;
pub mod wallet;
pub mod id_message;




