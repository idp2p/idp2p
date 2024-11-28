
use dotenv::dotenv;

use handler::IdMessageHandler;
use structopt::StructOpt;
use std::{error::Error, sync::Arc};
use tokio::{io::AsyncBufReadExt, select};

mod network;
mod utils;
mod handler;

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
    let mut handler = Arc::new(IdMessageHandler::new(opt.port)?);

    let mut stdin = tokio::io::BufReader::new(tokio::io::stdin()).lines();

    loop {
        select! {
            Ok(Some(line)) = stdin.next_line() => {
                let input: Vec<&str> = line.split(" ").collect();
                match input[0]{
                    "resolve" => {
                        Arc::get_mut(&mut handler).unwrap().resolve(input[1]);
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
            
        }
    }
}
