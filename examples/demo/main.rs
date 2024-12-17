use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

use cid::Cid;
use futures::channel::mpsc;
use idp2p_common::{cbor, cid::CidExt};
use idp2p_p2p::{
    handler::IdMessageHandler,
    store::{InMemoryKvStore, KvStore},
};
use network::IdNetworkEventLoop;
use serde::{Deserialize, Serialize};
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
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct IdUser {
    name: String,
    id: Option<Cid>,
}

impl IdUser {
    pub fn from_bytes(bytes: &[u8]) -> Self {
        let user: IdUser = cbor::decode(bytes).unwrap();
        user
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        cbor::encode(self).unwrap()
    }
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
    store.put("/users/alice", &alice.to_bytes())?;
    store.put("/users/bob", &bob.to_bytes())?;
    store.put("/users/dog", &dog.to_bytes())?;
    let (app_in_event_sender, app_in_event_receiver) = mpsc::channel(0);
    let (app_out_event_sender, app_out_event_receiver) = mpsc::channel(0);
    let id_handler = Arc::new(IdMessageHandler::new(store.clone())?);
    let (peer, network) = IdNetworkEventLoop::new(
        opt.name.clone(),
        store.clone(),
        app_in_event_sender.clone(),
        app_out_event_receiver,
        id_handler.clone(),
    )?;
    let id = utils::generate_id(&peer)?;
    let user_key = format!("/users/{}", &opt.name);
    let user = store.get(&user_key).unwrap().unwrap();
    let mut user: IdUser = cbor::decode(&user).unwrap();
    user.id = Some(Cid::from_bytes(&id.id).unwrap()); 
    store.put(&user_key, &user.to_bytes()).unwrap();
    tokio::spawn(network.run());
    app::run(opt.name.clone(), store.clone(), app_out_event_sender, app_in_event_receiver)
        .await
        .unwrap();
    Ok(())
}
