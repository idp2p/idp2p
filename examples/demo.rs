use std::collections::HashMap;

use idp2p_common::store::IdStore;
use structopt::StructOpt;
use tokio::io::{self, AsyncBufReadExt, BufReader};

bindgen!({
    path: "core/p2p/wit",
});

#[derive(Debug, StructOpt)]
#[structopt(name = "idp2p", about = "Usage of idp2p.")]
struct Opt {
    #[structopt(short = "p", long = "port", default_value = "43727")]
    port: u16,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv::dotenv().ok();
    env_logger::init();
    let opt = Opt::from_args();
    let id_store = idp2p_common::store::InMemoryIdStore::new();
    let p2p_component: Vec<u8> = id_store.get("/wasm/p2p/1.0.0")?.unwrap();
    let peer_addr = format!("/ip4/127.0.0.1/tcp/{}", opt.port);
    let mut stdin = BufReader::new(io::stdin()).lines();
    loop {
        tokio::select! {
            line = stdin.next_line() => {
                let input = line?.unwrap();
            }
        }
    }
}
use idp2p_common::cbor::decode;
use libp2p::gossipsub::Event as GossipsubEvent;
use serde::{Deserialize, Serialize};
use wasmtime::component::{bindgen, Component};
#[derive(Debug, Serialize, Deserialize)]
pub struct GossipMessage {
    pub version: String,
    pub data: Vec<u8>,
}
struct MyState;

pub fn handle_gossip_event(event: GossipsubEvent) -> anyhow::Result<()>{
    let id_store = idp2p_common::store::InMemoryIdStore::new();
    let engine = wasmtime::Engine::new(wasmtime::Config::new().wasm_component_model(true))?;
    let components: HashMap<String, Component> = HashMap::new();
    match event {
        GossipsubEvent::Message {
            propagation_source,
            message_id,
            message,
        } => {
            if message.topic.as_str().len() == 0 {
                let message: GossipMessage = decode(message.data.as_slice())?;
                let p2p_component: Vec<u8> = id_store.get(format!("/components/{}", message.version).as_str())?.unwrap();
                let component = wasmtime::component::Component::from_binary(&engine, &p2p_component)?;
                let mut store = wasmtime::Store::new(&engine, MyState {});
                //let linker = wasmtime::component::Linker::new(&engine);
                //let (idp2p, _instance) = Idp2pP2p::instantiate(&mut store, &components.get("k").unwrap(), &linker)?;
                
            }   
        }
        _ => {}
    }

    Ok(())
}
// Generate bindings of the guest and host components.
/*bindgen!({
    path: "../identity",
    additional_derives: [PartialEq, Eq, Hash, Clone, serde::Serialize, serde::Deserialize],
});*/
