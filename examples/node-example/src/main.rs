use crate::commands::get_command;
use crate::commands::handle_jwm;
use crate::store::FileStore;
use dotenv::dotenv;
use idp2p_common::anyhow::Result;
use idp2p_node::behaviour::IdentityEvent;
use idp2p_node::store::IdStore;
use idp2p_node::swarm::create_swarm;
use idp2p_node::swarm::SwarmOptions;
use libp2p::futures::StreamExt;
use libp2p::swarm::SwarmEvent;
use libp2p::Multiaddr;
use libp2p::PeerId;
use std::error::Error;
use structopt::StructOpt;
use tokio::io::{self, AsyncBufReadExt};
use tokio::sync::mpsc::channel;

#[derive(Debug, StructOpt)]
#[structopt(name = "idp2p", about = "Usage of idp2p node.")]
struct Opt {
    #[structopt(short = "p", long = "port", default_value = "43727")]
    port: u16,
    #[structopt(
        short = "r",
        long = "rendezvous_point"
    )]
    rendezvous_point: Option<(String, String)>,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    dotenv().ok();
    let opt = Opt::from_args();
    let mut stdin = io::BufReader::new(io::stdin()).lines();
    let (tx, mut rx) = channel::<IdentityEvent>(100);
    let options = SwarmOptions {
        port: opt.port,
        store: IdStore::new(tx.clone()),
    };
    let mut swarm = create_swarm(options).await?;
    let rendezvous_address = opt.rendezvous_address.parse::<Multiaddr>().unwrap();
    let rendezvous_point = opt.rendezvous_id.parse::<PeerId>().unwrap();
    swarm.dial(rendezvous_address.clone()).unwrap();
    
    loop {
        tokio::select! {
            line = stdin.next_line() => {
                let line = line?.expect("stdin closed");
            }
            swarm_event = swarm.select_next_some() => {
                match swarm_event {
                    SwarmEvent::NewListenAddr { address, .. } => {
                        println!("Listening on {:?}", address);
                    }
                    SwarmEvent::ConnectionEstablished{peer_id, ..} => {
                        println!("Established {:?}", peer_id);
                    }
                    other => {
                        println!("Unhandled {:?}", other);
                    }
                }
            }
            event = rx.recv() => {
                if let Some(event) = event{
                    match event{
                        IdentityEvent::ReceivedJwm {id, jwm} => {
                            let mes = handle_jwm(&id, &jwm, swarm.behaviour_mut())?;
                            println!("{mes}");
                        }
                        IdentityEvent::Discovered {addr} => {
                            swarm.dial(addr).unwrap();
                        }
                        _ => println!("{:?}", event)
                    }
                }
            }
        }
    }
}

pub mod commands;
pub mod store;
