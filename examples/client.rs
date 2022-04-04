use idp2p_client::behaviour::{IdGossipBehaviour, IdMessagePublisher, IdentityClientEvent};
use idp2p_client::commands::IdCommand;
use idp2p_client::file::FilePersister;
use idp2p_client::swarm::{build_swarm, IdSwarmOptions};
use idp2p_client::IdConfigResolver;
use idp2p_core::{IdProfile, IdentityEvent};
use idp2p_wallet::store::WalletStore;
use libp2p::futures::StreamExt;
use libp2p::swarm::SwarmEvent;
use std::str::FromStr;
use std::sync::Arc;
use structopt::StructOpt;
use tokio::io::AsyncBufReadExt;
use tokio::sync::mpsc::channel;

#[derive(Debug, StructOpt)]
#[structopt(name = "idp2p", about = "Usage of idp2p.")]
struct Opt {
    #[structopt(short = "p", long = "port", default_value = "43727")]
    port: u16,
    #[structopt(short = "r", long = "remote")]
    remote: Option<String>,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let opt = Opt::from_args();
    let (tx, mut rx) = channel::<IdentityEvent>(10);
    let mut stdin = tokio::io::BufReader::new(tokio::io::stdin()).lines();
    let persister = FilePersister::from_str("./target")?;
    let config = persister.get_config(opt.port, opt.remote)?;
    let swarm_opt = IdSwarmOptions::new(config, tx.clone());
    let mut swarm = build_swarm(swarm_opt).await?;
    let wallet_store = Arc::new(WalletStore::new(persister));
    loop {
        tokio::select! {
            line = stdin.next_line() => {
                let line = line?.expect("stdin closed");
                if let Some(cmd) = get_command(&line){
                    cmd.handle(wallet_store.clone())?;
                }
            }
            swarm_event = swarm.select_next_some() => {
                match swarm_event {
                    SwarmEvent::NewListenAddr { address, .. } => {
                        println!("Listening on {:?}", address);
                    }
                    SwarmEvent::Behaviour(IdentityClientEvent::Gossipsub(event)) =>{
                        let store = swarm.behaviour().id_store.clone();
                        swarm.behaviour_mut().gossipsub.handle_event(event, store).await;
                    }
                    SwarmEvent::Behaviour(IdentityClientEvent::Mdns(event)) =>{
                        swarm.behaviour_mut().handle_mdns_event(event);
                    }
                    other => {println!("{:?}", other);}
                }
            }
            event = rx.recv() => {
                if let Some(event) = event{
                    match event{
                        IdentityEvent::Publish { id } => {
                            let store = swarm.behaviour().id_store.clone();
                            swarm.behaviour_mut().gossipsub.publish_msg(&id, store)?;
                        }

                        _ => { /* persist */  }
                    }
                }
            }
        }
    }
}

fn get_command(input: &str) -> Option<IdCommand> {
    let split = input.split(" ");
    let input: Vec<&str> = split.collect();
    match input[0] {
        "register" => {
            let profile = IdProfile::new(input[1], &vec![]);
            let cmd = IdCommand::Register {
                profile: profile,
                password: input[2].to_owned(),
            };
            return Some(cmd);
        }
        "get" => {
            return Some(IdCommand::Get);
        }
        _ => println!("Unknown command"),
    }
    None
}
