use idp2p_node::store::IdStore;
use crate::store::FileStore;
use dotenv::dotenv;
use idp2p_common::anyhow::Result;
use idp2p_node::behaviour::IdentityGossipEvent;
use idp2p_node::swarm::create_swarm;
use idp2p_node::swarm::SwarmOptions;
use idp2p_wallet::wallet::CreateAccountResult;
use idp2p_wallet::wallet::Wallet;
use libp2p::futures::StreamExt;
use libp2p::swarm::SwarmEvent;
use std::error::Error;
use structopt::StructOpt;
use tokio::io::{self, AsyncBufReadExt};
use tokio::sync::mpsc::channel;

#[derive(Debug, StructOpt)]
#[structopt(name = "idp2p", about = "Usage of idp2p.")]
struct Opt {
    #[structopt(short = "i", long = "id")]
    id: String,
}

pub fn init(name: &str) -> Result<(u16, CreateAccountResult)> {
    let base_path = format!("../target/{}", name);
    std::env::set_var("BASE_PATH", base_path.clone());
    let id_path = format!("{}/identities", base_path);
    std::fs::create_dir_all(id_path).unwrap();
    let acc_path = format!("{}/accounts", base_path);
    std::fs::create_dir_all(acc_path).unwrap();
    let mut port = 5000;
    if name == "bob" {
        port = 6000;
    }
    let seed = idp2p_common::create_random::<16>();
    let password = "123456";
    let wallet = Wallet::new(password)?;
    let payload = wallet.get_payload(password)?;
    let result = payload.create_account(name, seed)?;
    Ok((port, result))
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    dotenv().ok();
    let opt = Opt::from_args();
    let (port, acc_result) = init(&opt.id)?;
    let mut stdin = io::BufReader::new(io::stdin()).lines();
    let (tx, mut rx) = channel::<IdentityGossipEvent>(100);
    let options = SwarmOptions {
        port: port,
        sender: tx.clone(),
    };
    let mut swarm = create_swarm(options).await?;
    let did = acc_result.did.clone();
    FileStore{}.put(&did.id.clone(), did.clone());
    swarm.behaviour_mut().subscribe(did.id);
    loop {
        tokio::select! {
            line = stdin.next_line() => {
                let line = line?.expect("stdin closed");
                println!("{}", line);
            }
            listen_event = swarm.select_next_some() => {
                if let SwarmEvent::NewListenAddr { address, .. } = listen_event {
                    println!("Listening on {:?}", address);
                }
            }
            event = rx.recv() => {
                if let Some(event) = event{
                    println!("{:?}", event);
                    let id_store = FileStore{};
                    let ids = &mut swarm.behaviour_mut().identities;
                    let result = event.message.handle(&event.topic, ids, id_store);
                    match result{
                        Ok(r) => println!("{:?}", r),
                        Err(_) => println!("Err")
                    }
                }
            }
        }
    }
}

pub mod commands;
pub mod store;
