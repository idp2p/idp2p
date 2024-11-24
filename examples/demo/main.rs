use behaviour::{create_swarm, Idp2pBehaviourEvent};
use dotenv::dotenv;
use futures::channel::mpsc::*;
use futures::StreamExt;

use idp2p_p2p::{
    message::IdMessageHandler,
    store::{InMemoryKvStore, KvStore},
};
use libp2p::swarm::SwarmEvent;
use structopt::StructOpt;
use std::{error::Error, sync::Arc};
use tokio::{io::AsyncBufReadExt, select};

mod behaviour;
mod utils;

#[derive(Debug, StructOpt)]
#[structopt(name = "idp2p", about = "Usage of idp2p.")]
struct Opt {
    #[structopt(short = "p", long = "port", default_value = "43727")]
    port: u16,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    dotenv().ok();
    env_logger::init();
    let opt = Opt::from_args();

    let kv = Arc::new(InMemoryKvStore::new());
    let (sender, _) = channel(100);
    let mut handler = IdMessageHandler::new(kv.clone(), sender)?;

    let mut swarm = create_swarm(opt.port)?;
    let mut stdin = tokio::io::BufReader::new(tokio::io::stdin()).lines();
    let id = utils::generate_id(swarm.local_peer_id())?;
    println!("Id {}", id.id.to_string());
    loop {
        select! {
            Ok(Some(line)) = stdin.next_line() => {
                let input: Vec<&str> = line.split(" ").collect();
                match input[0]{
                    "resolve" => {
                        println!("Resolve {}", input[1]);
                    },
                    "upgrade" => {
                        println!("Upgrade");
                    },
                    "send_message" => {
                        println!("Send message to {} with {}", input[1], input[2]);
                    }
                    _ => println!("Unknown command")
                }
            }
            event = swarm.select_next_some() => match event {

                SwarmEvent::Behaviour(Idp2pBehaviourEvent::Mdns(libp2p::mdns::Event::Discovered(list))) => {
                    for (peer_id, _multiaddr) in list {
                        println!("mDNS discovered a new peer: {peer_id}");
                        swarm.behaviour_mut().gossipsub.add_explicit_peer(&peer_id);
                    }
                },
                SwarmEvent::Behaviour(Idp2pBehaviourEvent::Mdns(libp2p::mdns::Event::Expired(list))) => {
                    for (peer_id, _multiaddr) in list {
                        println!("mDNS discover peer has expired: {peer_id}");
                        swarm.behaviour_mut().gossipsub.remove_explicit_peer(&peer_id);
                    }
                },
                SwarmEvent::Behaviour(Idp2pBehaviourEvent::Gossipsub(event)) => {
                     handler.handle_gossip_message(event).await?;
                },
                SwarmEvent::Behaviour(Idp2pBehaviourEvent::RequestResponse(event)) => {
                    todo!()
                }
                SwarmEvent::NewListenAddr { address, .. } => {
                    println!("Local node is listening on {address}");
                },
                _ => {}
            }
        }
    }
}
