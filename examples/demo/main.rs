use futures::{channel::mpsc, SinkExt, StreamExt};
use idp2p_common::id::Id;
use idp2p_p2p::{
    handler::{IdMessageHandler, IdMessageHandlerCommand},
    verifier::IdVerifierImpl,
};
use network::{IdNetworkCommand, IdNetworkEventLoop, IdRequestKind};
use std::{fs, sync::Arc};
use store::{InMemoryIdStore, InMemoryKvStore};
use structopt::StructOpt;
use tracing_subscriber::EnvFilter;
use user::UserState;

mod app;
mod network;
mod store;
mod user;
mod utils;

#[derive(Debug, StructOpt)]
#[structopt(name = "idp2p", about = "Usage of idp2p.")]
struct Opt {
    #[structopt(short = "i", long = "id")]
    name: String,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenv::dotenv().ok();
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
    let comp_bytes = fs::read("./target/wasm32-unknown-unknown/debug/idp2p_id.wasm")
        .expect("Failed to read input file");
    let comp_id = Id::new("component", 0x01, &comp_bytes).unwrap().to_string();
    let comps = [(comp_id.clone(), comp_bytes)].into_iter().collect();
    let verifier = Arc::new(IdVerifierImpl::new(comps)?);
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
    let pid = utils::generate_actor(&comp_id, &peer)?;
    let user = UserState::new(&opt.name, &pid.id, &peer.to_string());
    store.set_current_user(&user).await.unwrap();
    tokio::spawn(network.run());
    
   
    tokio::spawn({
        let mut network_cmd_sender_clone = network_cmd_sender.clone();
        
        let mut handler_cmd_receiver = handler_cmd_receiver;
        async move {
            loop {
                tokio::select! {
                    handler_cmd = handler_cmd_receiver.next() => match handler_cmd {
                        Some(cmd) => match cmd {
                            IdMessageHandlerCommand::Request { peer, payload } => {
                                let req = IdRequestKind::Message(payload);
                                network_cmd_sender_clone.send(IdNetworkCommand::SendRequest {
                                    peer,
                                    req
                                }).await.unwrap();
                            }
                            IdMessageHandlerCommand::Publish { topic, payload } => {
                                network_cmd_sender_clone.send(IdNetworkCommand::Publish {
                                    topic,
                                    payload
                                }).await.unwrap();
                            }
                        },
                        None => break
                    },
                }
            }
        }
    });
    app::run(store, network_cmd_sender, app_event_receiver).await?;
    Ok(())
}
