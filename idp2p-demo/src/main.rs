use crate::account::Account;
use dotenv::dotenv;
use idp2p_node::message::IdentityMessageResult;
use idp2p_node::swarm::create_swarm;
use libp2p::futures::StreamExt;
use libp2p::swarm::SwarmEvent;
use std::error::Error;
use structopt::StructOpt;
use tokio::io::{self, AsyncBufReadExt};

#[derive(Debug, StructOpt)]
#[structopt(name = "idp2p", about = "Usage of idp2p.")]
struct Opt {
    #[structopt(short = "i", long = "id")]
    id: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    dotenv().ok();
    let opt = Opt::from_args();
    let port = Account::init(&opt.id);
    let mut stdin = io::BufReader::new(io::stdin()).lines();
    let (sender, mut receiver) = tokio::sync::mpsc::channel::<IdentityMessageResult>(100);
    let mut swarm = create_swarm(port, sender.clone()).await?;
    //let cmd_sender = sender.clone();
    loop {
        tokio::select! {
            line = stdin.next_line() => {
                let line = line?.expect("stdin closed");
                println!("{}", line);
            }
            event = swarm.select_next_some() => {
                if let SwarmEvent::NewListenAddr { address, .. } = event {
                    println!("Listening on {:?}", address);
                }
            }
            message = receiver.recv() => {
                if let Some(message) = message{
                    println!("{:?}", message);
                    //message.handle(swarm.behaviour_mut());
                }
            }
        }
    }
}

pub mod account;
