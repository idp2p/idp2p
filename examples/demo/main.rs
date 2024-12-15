use std::{collections::HashMap, sync::{Arc, Mutex}};

use cid::Cid;
use futures::channel::mpsc;
use idp2p_common::cid::CidExt;
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
    name: String
}

pub struct IdDemo {
  pub users: HashMap<String, IdUser>,
  pub current: String   
}

#[derive(Debug, Clone)]
pub struct IdUser {
    name: String, 
    port: u16,
    id: Option<Cid>,
}

impl IdDemo {
    pub fn new(current: String) -> IdDemo {
        let alice = IdUser { name: "Alice".to_string(), port: 43727, id: None };
        let bob = IdUser { name: "Bob".to_string(), port: 43728, id: None };
        let dog = IdUser { name: "Dog".to_string(), port: 43729, id: None };

        let mut users = HashMap::new();
        users.insert("alice".to_string(), alice);
        users.insert("bob".to_string(), bob);
        users.insert("dog".to_string(), dog);
        IdDemo {
            users: users,
            current: current,
        }
    }

    pub fn get_current_user(&self) -> &IdUser {
        self.users.get(&self.current).unwrap()
    }

    pub fn set_id(&mut self, user: &str, id: &[u8]) {
        let id = Some(Cid::from_bytes(id).unwrap());
        let mut user = self.users.get_mut(user).unwrap().clone();
        user.id = id;
        self.users.insert(user.name.clone(), user);
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
    let demo = Arc::new(Mutex::new(IdDemo::new(opt.name.clone()))); 
    let store = Arc::new(InMemoryKvStore::new());
    let (app_in_event_sender, app_in_event_receiver) = mpsc::channel(0);
    let (app_out_event_sender, app_out_event_receiver) = mpsc::channel(0);
    let id_handler = Arc::new(IdMessageHandler::new(store.clone())?);
    let (peer, network) = IdNetworkEventLoop::new(
        demo.clone(),
        app_in_event_sender.clone(),
        app_out_event_receiver,
        id_handler.clone()
    )?;
    let id = utils::generate_id(&peer)?;
    demo.lock().unwrap().set_id(&opt.name, &id.id);
    tokio::spawn(network.run());
    app::run(opt.name, app_out_event_sender, app_in_event_receiver).await.unwrap();
    Ok(())
}
