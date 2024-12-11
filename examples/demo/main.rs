use std::sync::Arc;

use futures::{channel::mpsc, SinkExt};
use idp2p_p2p::store::InMemoryKvStore;
use network::IdNetworkEventLoop;
use structopt::StructOpt;
use tracing_subscriber::EnvFilter;

mod app;
mod network;
mod utils;

#[derive(Debug, StructOpt)]
#[structopt(name = "idp2p", about = "Usage of idp2p.")]
struct Opt {
    #[structopt(short = "p", long = "port", default_value = "43727")]
    port: u16,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let opt = Opt::from_args();
    color_eyre::install().map_err(anyhow::Error::msg)?;
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .try_init().unwrap();
    let store = Arc::new(InMemoryKvStore::new());
    let (network_cmd_sender, network_cmd_receiver) = mpsc::channel(0);
    let (handler_cmd_sender, handler_cmd_receiver) = mpsc::channel(0);
    let network =
        IdNetworkEventLoop::new(opt.port, store.clone(), handler_cmd_sender, network_cmd_receiver)?;
    tokio::spawn(network.run());

    //let handler = IdMessageHandler::new(store.clone(), network_cmd_sender, handler_cmd_receiver)?;
    //tokio::spawn(handler.run(&mut handler_cmd_receiver));
    let mut stdin = tokio::io::AsyncBufReadExt::lines(tokio::io::BufReader::new(tokio::io::stdin()));

    loop {
        tokio::select! {
            Ok(Some(line)) = stdin.next_line() => {
                let input: Vec<&str> = line.split(" ").collect();
                match input[0]{
                    "resolve" => {
                        println!("Resolve {}", input[1]);
                        //network_event_sender.send(item);
                        // publish resolve message
                    },
                    "mutate" => {
                        println!("Mutate");
                        // publish mutate message
                    },
                    "send_message" => {
                        println!("Send message to {} with {}", input[1], input[2]);
                        // publish message
                    }
                    _ => println!("Unknown command")
                }
            }

        }
    }
}
