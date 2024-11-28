use std::sync::Arc;

use app::App;
use dotenv::dotenv;

use futures::channel::mpsc;
use network::IdNetworkEventLoop;
use store::InMemoryKvStore;
use structopt::StructOpt;

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
    dotenv().ok();
    color_eyre::install().map_err(anyhow::Error::msg)?;
    env_logger::init();
    let opt = Opt::from_args();
    let store = Arc::new(InMemoryKvStore::new());
    let (network_event_sender, network_event_receiver) = mpsc::channel(0);
    let (wasm_event_sender, wasm_event_receiver) = mpsc::channel(0);
    let network =
        IdNetworkEventLoop::new(opt.port, store, wasm_event_sender, network_event_receiver)?;
    // create message handler
    // create network swarm
    // create gui app and start
    let terminal = ratatui::init();
    let app_result = App::new().run(terminal);
    ratatui::restore();
    app_result

    /*let mut handler = Arc::new(IdMessageHandler::new(opt.port)?);

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
    }*/
}
