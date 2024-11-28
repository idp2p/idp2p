use std::sync::Arc;

use app::App;

use futures::{channel::mpsc, SinkExt};
use network::IdNetworkEventLoop;
use store::InMemoryKvStore;
use structopt::StructOpt;
use tracing_subscriber::EnvFilter;

mod app;
mod event;
mod handler;
mod network;
mod store;
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
    let _ = tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .try_init();
    let store = Arc::new(InMemoryKvStore::new());
    let (network_event_sender, network_event_receiver) = mpsc::channel(0);
    let (wasm_event_sender, wasm_event_receiver) = mpsc::channel(0);
    let mut network =
        IdNetworkEventLoop::new(opt.port, store, wasm_event_sender, network_event_receiver)?;
    tokio::spawn(network.run());
    // create message handler
    // create network swarm
    // create gui app and start
    /*let terminal = ratatui::init();
    let app_result = App::new().run(terminal);
    ratatui::restore();
    app_result*/

    /*let mut handler = Arc::new(IdMessageHandler::new(opt.port)?);*/

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
