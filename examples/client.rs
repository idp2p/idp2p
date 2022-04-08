use idp2p_client::behaviour::*;
use idp2p_client::commands::IdCommand;
use idp2p_client::commands::IdCommandHandler;
use idp2p_client::file_db::FilePersister;
use idp2p_client::swarm::*;
use idp2p_client::IdConfigResolver;
use idp2p_core::IdentityEvent;
use idp2p_wallet::store::WalletStore;
use libp2p::futures::StreamExt;
use libp2p::swarm::SwarmEvent;
use std::str::FromStr;
use std::sync::Arc;
use std::time::Duration;
use structopt::StructOpt;
use tokio::io::AsyncBufReadExt;
use tokio::sync::mpsc::channel;
use tokio::time::sleep;

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
    dotenv::dotenv().ok();
    env_logger::init();
    let opt = Opt::from_args();
    let (tx, mut rx) = channel::<IdentityEvent>(10);
    let mut stdin = tokio::io::BufReader::new(tokio::io::stdin()).lines();
    let path = format!("./target/{}", opt.port);
    let persister = FilePersister::from_str(&path)?;
    let config = persister.get_or_save_config("0.0.0.0", opt.port, opt.remote)?;
    let swarm_opt = IdSwarmOptions::new(config, tx.clone());
    let mut swarm = build_swarm(swarm_opt).await?;
    let wallet_store = Arc::new(WalletStore::new(persister));
    let command_handler = IdCommandHandler {
        wallet_store: wallet_store.clone(),
        id_store: swarm.behaviour_mut().id_store.clone(),
        event_sender: tx.clone(),
    };
    loop {
        tokio::select! {
            line = stdin.next_line() => {
                let line = line?.expect("stdin closed");
                if let Some(cmd) = get_command(&line){
                    let is_get = cmd == IdCommand::Get;
                    let result = command_handler.handle(cmd).await?;
                    if is_get{
                        idp2p_common::log::info!("{}", idp2p_common::serde_json::to_string(&result)?);
                    }
                }
            }
            swarm_event = swarm.select_next_some() => {
                match swarm_event {
                    SwarmEvent::NewListenAddr { address, .. } => {
                        idp2p_common::log::info!("Listening on {:?}", address);
                    }
                    SwarmEvent::Behaviour(IdentityClientEvent::Gossipsub(event)) =>{
                        command_handler.handle_gossip_event(event).await?;
                    }
                    SwarmEvent::Behaviour(IdentityClientEvent::Mdns(event)) =>{
                        swarm.behaviour_mut().handle_mdns_event(event);
                    }
                    other => idp2p_common::log::info!("{:?}", other)
                }
            }
            event = rx.recv() => {
                if let Some(event) = event{
                    match event{
                        IdentityEvent::GetHandled { id } => {
                            let store = swarm.behaviour().id_store.clone();
                            swarm.behaviour_mut().gossipsub.publish_msg(&id, store)?;
                        }
                        IdentityEvent::Connected { id } => {
                            swarm.behaviour_mut().gossipsub.subscribe_to(&id)?;
                            sleep(Duration::from_secs(2)).await;
                            swarm.behaviour_mut().gossipsub.publish_get(&id)?;
                        }
                        IdentityEvent::Created { id } => {
                            swarm.behaviour_mut().gossipsub.subscribe_to(&id)?;
                        }
                        IdentityEvent::JwmReceived { jwm } => {
                            //
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
            let cmd = IdCommand::Register {
                name: input[1].to_owned(),
                photo: vec![],
                password: input[2].to_owned(),
            };
            return Some(cmd);
        }
        "login" => {
            return Some(IdCommand::Login {
                password: input[1].to_owned(),
            });
        }
        "connect" => {
            return Some(IdCommand::Connect {
                id: input[1].to_owned(),
            });
        }
        "get" => {
            return Some(IdCommand::Get);
        }
        _ => println!("Unknown command"),
    }
    None
}
