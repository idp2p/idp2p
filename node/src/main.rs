use crate::commands::get_command;
use crate::id_swarm::create;
use libp2p::futures::StreamExt;
use libp2p::swarm::SwarmEvent;
use libp2p::Multiaddr;
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
    if let Some(to_dial) = std::env::args().nth(1) {
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
        }
    }
}

pub mod behaviour;
pub mod commands;
pub mod id_message;
pub mod id_swarm;
pub mod wallet;
