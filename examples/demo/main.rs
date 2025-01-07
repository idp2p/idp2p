use std::{collections::HashMap, sync::Arc};

use futures::{channel::mpsc, StreamExt};
use idp2p_p2p::{handler::IdMessageHandler, verifier::IdVerifierImpl};
use network::IdNetworkEventLoop;
use store::{InMemoryIdStore, InMemoryKvStore};
use structopt::StructOpt;
use tracing_subscriber::EnvFilter;
use user::UserState;

mod app;
mod network;
mod store;
mod utils;
mod user;

#[derive(Debug, StructOpt)]
#[structopt(name = "idp2p", about = "Usage of idp2p.")]
struct Opt {
    #[structopt(short = "i", long = "id")]
    name: String,
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
    
    let (handler_cmd_sender, handler_cmd_receiver) = mpsc::channel(0);
    let (app_event_sender, app_event_receiver) = mpsc::channel(0);
    let (network_cmd_sender, network_cmd_receiver) = mpsc::channel(0);
    let verifier = Arc::new(IdVerifierImpl::new(HashMap::new())?);
    let id_store = Arc::new(InMemoryIdStore(store.clone()));
    let id_handler = IdMessageHandler::new(
        id_store.clone(),
        verifier.clone(),
        handler_cmd_sender.clone(),
    );
    let port: u16 = match opt.name.as_str() {
        "alice" => 43727,
        "bob" => 43728,
        "dog" => 43729,
        _ => panic!("Unknown user"),
    };
    let (peer, network) = IdNetworkEventLoop::new(
        port,
        store.clone(),
        app_event_sender.clone(),
        network_cmd_receiver,
        id_handler,
    )?;
    let (id, pid) = utils::generate_id(&peer)?;
    tokio::spawn(network.run());
    let user = UserState::new(&opt.name, &id, &peer.to_string());
    tokio::spawn({
        let mut handler_cmd_receiver = handler_cmd_receiver;
        async move {
            loop {
                tokio::select! {
                    handler_cmd = handler_cmd_receiver.next() => todo!(),
                }
            }
        }
    });
    app::run(
        store.clone(),
        network_cmd_sender,
        app_event_receiver,
    )
    .await
    .unwrap();
    Ok(())
}
