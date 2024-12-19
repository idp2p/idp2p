use std::{collections::HashMap, sync::Arc};

use cid::Cid;
use futures::{channel::mpsc, StreamExt};
use idp2p_common::cid::CidExt;
use idp2p_p2p::{handler::IdMessageHandler, verifier::IdVerifierImpl};
use network::IdNetworkEventLoop;
use serde::{Deserialize, Serialize};
use store::{InMemoryIdStore, InMemoryKvStore};
use structopt::StructOpt;
use tracing_subscriber::EnvFilter;

mod app;
mod network;
mod store;
mod utils;

#[derive(Debug, StructOpt)]
#[structopt(name = "idp2p", about = "Usage of idp2p.")]
struct Opt {
    #[structopt(short = "i", long = "id")]
    name: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct IdUser {
    name: String,
    id: Option<Cid>,
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
    let alice = IdUser {
        name: "Alice".to_string(),
        id: None,
    };
    let bob = IdUser {
        name: "Bob".to_string(),
        id: None,
    };
    let dog = IdUser {
        name: "Dog".to_string(),
        id: None,
    };
    store.set_user("alice", &alice).await?;
    store.set_user("bob", &bob).await?;
    store.set_user("dog", &dog).await?;
    let (handler_cmd_sender, handler_cmd_receiver) = mpsc::channel(0);
    let (app_event_sender, app_event_receiver) = mpsc::channel(0);
    let (network_cmd_sender, network_cmd_receiver) = mpsc::channel(0);
    let verifier = Arc::new(IdVerifierImpl::new(HashMap::new())?);
    let id_store = Arc::new(InMemoryIdStore(store.clone()));
    let id_handler = IdMessageHandler::new(
        id_store.clone(),
        verifier.clone(),
        handler_cmd_sender.clone(),
    )?;
    let (peer, network) = IdNetworkEventLoop::new(
        opt.name.clone(),
        store.clone(),
        app_event_sender.clone(),
        network_cmd_receiver,
        id_handler,
    )?;
    let id = utils::generate_id(&peer)?;
    let mut user = store.get_user(&opt.name).await.unwrap().unwrap();
    user.id = Some(Cid::from_bytes(&id.id).unwrap());
    store.set_user(&opt.name, &user).await.unwrap();
    tokio::spawn(network.run());

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
        opt.name.clone(),
        peer,
        store.clone(),
        network_cmd_sender,
        app_event_receiver,
    )
    .await
    .unwrap();
    Ok(())
}
