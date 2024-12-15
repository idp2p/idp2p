use std::sync::Arc;

use futures::channel::mpsc;
use idp2p_p2p::{handler::IdMessageHandler, store::InMemoryKvStore};
use network::IdNetworkEventLoop;
use structopt::StructOpt;
use tracing_subscriber::EnvFilter;

mod app;
mod network;
mod utils;

#[derive(Debug, StructOpt)]
#[structopt(name = "idp2p", about = "Usage of idp2p.")]
struct Opt {
    #[structopt(short = "i", long = "id")]
    name: String,
    #[structopt(short = "p", long = "port", default_value = "43727")]
    port: u16,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let opt = Opt::from_args();
    color_eyre::install().map_err(anyhow::Error::msg)?;
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .try_init()
        .unwrap();
    let store = Arc::new(InMemoryKvStore::new());
    let (handler_outbound_event_sender, handler_outbound_event_receiver) = mpsc::channel(0);
    let (handler_inbound_event_sender, handler_inbound_event_receiver) = mpsc::channel(0);
    let (app_event_sender, app_event_receiver) = mpsc::channel(0);
    let handler = IdMessageHandler::new(
        store.clone(),
        handler_outbound_event_sender,
        handler_inbound_event_receiver,
    )?;
    tokio::spawn(handler.run());
    let network = IdNetworkEventLoop::new(
        opt.port,
        app_event_sender,
        handler_inbound_event_sender.clone(),
        handler_outbound_event_receiver,
    )?;
    tokio::spawn(network.run());
    app::run(opt.name, handler_inbound_event_sender, app_event_receiver).await.unwrap();
    Ok(())
}
