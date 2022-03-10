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
#[structopt(name = "idp2p", about = "Usage of idp2p.")]
struct Opt {
    #[structopt(short = "p", long = "port", default_value = "43727")]
    port: u16,
    #[structopt(short = "a", long = "addr", default_value = "0.0.0.0")]
    address: String,
    #[structopt(
        short = "r",
        long = "r_addr",
        default_value = "/ip4/127.0.0.1/tcp/43727"
    )]
    rendezvous_address: String,
    #[structopt(
        short = "i",
        long = "r_id",
        default_value = "12D3KooWDpJ7As7BWAwRMfu1VU2WCqNjvq387JEYKDBj4kx6nXTN"
    )]
    rendezvous_id: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    dotenv().ok();
    let opt = Opt::from_args();
    let base_path = format!("../target/{}", opt.port);
    std::env::set_var("BASE_PATH", base_path.clone());
    let acc_path = format!("{}/accounts", base_path);
    std::fs::create_dir_all(acc_path).unwrap();
    let mut stdin = io::BufReader::new(io::stdin()).lines();
    let (tx, mut rx) = channel::<IdentityEvent>(100);
    let options = SwarmOptions {
        addr: opt.address,
        port: opt.port,
        store: IdStore::new(tx.clone()),
    };
    let mut swarm = create_swarm(options).await?;
    let rendezvous_address = opt.rendezvous_address.parse::<Multiaddr>().unwrap();
    let rendezvous_point = opt.rendezvous_id.parse::<PeerId>().unwrap();
    swarm.dial(rendezvous_address.clone()).unwrap();
    swarm
        .behaviour_mut()
        .auto_nat
        .add_server(rendezvous_point, Some(rendezvous_address));
    loop {
        tokio::select! {
            line = stdin.next_line() => {
                let line = line?.expect("stdin closed");
                if line == "Discover"{
                    swarm.behaviour_mut().rendezvous.discover(
                        Some(libp2p::rendezvous::Namespace::new("rendezvous".to_owned()).unwrap()),
                        None,
                        None,
                        rendezvous_point
                    );
                }
                if let Some(cmd) = get_command(&line){
                    cmd.handle(swarm.behaviour_mut())?;
                }
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
