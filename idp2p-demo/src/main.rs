use crate::commands::get_command;
use crate::commands::handle_jwm;
use crate::store::FileStore;
use dotenv::dotenv;
use idp2p_common::anyhow::Result;
use idp2p_node::behaviour::IdentityEvent;
use idp2p_node::swarm::create_swarm;
use idp2p_node::swarm::SwarmOptions;
use libp2p::futures::StreamExt;
use libp2p::swarm::SwarmEvent;
use libp2p::mdns::{ Mdns, MdnsEvent};
use std::error::Error;
use structopt::StructOpt;
use tokio::io::{self, AsyncBufReadExt};
use tokio::sync::mpsc::channel;

#[derive(Debug, StructOpt)]
#[structopt(name = "idp2p", about = "Usage of idp2p.")]
struct Opt {
    #[structopt(short = "p", long = "port", default_value = "43727")]
    port: u16,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    dotenv().ok();
    let opt = Opt::from_args();
    let base_path = format!("target/{}", opt.port);
    std::env::set_var("BASE_PATH", base_path.clone());
    let id_path = format!("{}/identities", base_path);
    std::fs::create_dir_all(id_path).unwrap();
    let acc_path = format!("{}/accounts", base_path);
    std::fs::create_dir_all(acc_path).unwrap();
    let mut stdin = io::BufReader::new(io::stdin()).lines();
    let (tx, mut rx) = channel::<IdentityEvent>(100);
    let options = SwarmOptions {
        port: opt.port,
        sender: tx.clone(),
        store: Box::new(FileStore {}),
    };
    let mut swarm = create_swarm(options).await?;
    loop {
        tokio::select! {
            line = stdin.next_line() => {
                let line = line?.expect("stdin closed");
                if let Some(cmd) = get_command(&line){
                    cmd.handle(swarm.behaviour_mut())?;
                }
            }
            swarm_event = swarm.select_next_some() => {
                match swarm_event {
                    SwarmEvent::NewListenAddr { address, .. } => {
                        println!("Listening on {:?}", address);
                    }
                    SwarmEvent::ConnectionEstablished{peer_id, ..}=> {
                            println!("discovered {}", peer_id);
                    }
                    _ => {}
                }
            }
            event = rx.recv() => {
                if let Some(event) = event{
                    match event{
                        IdentityEvent::ReceivedJwm { id, jwm} => {
                            let mes = handle_jwm(&id, &jwm)?;
                            println!("{mes}");
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
