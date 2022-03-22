use idp2p_node::node::build_gossipsub;
use idp2p_node::node::build_transport;
use idp2p_node::store::IdStore;
use idp2p_node::store::IdStoreOptions;
use idp2p_node::IdentityEvent;
use idp2p_wallet::store::WalletEvent;
use idp2p_wallet::store::WalletOptions;
use idp2p_wallet::store::WalletStore;
use libp2p::{
    futures::StreamExt,
    gossipsub::{Gossipsub, GossipsubEvent, IdentTopic},
    identity::Keypair,
    swarm::{SwarmBuilder, SwarmEvent},
    NetworkBehaviour,
};
use std::collections::HashMap;
use std::path::PathBuf;
use std::str::FromStr;

use tokio::io::AsyncBufReadExt;
use tokio::sync::mpsc::channel;

#[derive(NetworkBehaviour)]
#[behaviour(out_event = "IdentityNodeEvent")]
pub struct IdentityNodeBehaviour {
    pub gossipsub: Gossipsub,
    #[behaviour(ignore)]
    pub id_store: IdStore,
}

#[derive(Debug)]
pub enum IdentityNodeEvent {
    Gossipsub(GossipsubEvent),
}

impl From<GossipsubEvent> for IdentityNodeEvent {
    fn from(event: GossipsubEvent) -> Self {
        IdentityNodeEvent::Gossipsub(event)
    }
}

fn run_command(input: &str, store: WalletStore) {
    let split = input.split(" ");
    let input: Vec<&str> = split.collect();
    match input[0] {
        "get" => {
            let state =
                idp2p_common::serde_json::to_string_pretty(&store.get_state().unwrap()).unwrap();
            println!("{}", state);
        }
        _ => {}
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut stdin = tokio::io::BufReader::new(tokio::io::stdin()).lines();
    let (tx, mut rx) = channel::<IdentityEvent>(100);
    let local_key = Keypair::generate_ed25519();
    let transport = build_transport(local_key.clone()).await;
    let options = IdStoreOptions {
        event_sender: tx.clone(),
        entries: HashMap::new(),
    };
    let id_store = IdStore::new(options);
    let mut swarm = {
        let behaviour = IdentityNodeBehaviour {
            gossipsub: build_gossipsub(),
            id_store: id_store,
        };
        let executor = Box::new(|fut| {
            tokio::spawn(fut);
        });
        SwarmBuilder::new(transport, behaviour, local_key.public().to_peer_id())
            .executor(executor)
            .build()
    };
    let (w_tx, w_rx) = channel::<WalletEvent>(100);
    let wallet_opt = WalletOptions {
        wallet_path: PathBuf::from_str("./wallet.json")?,
        event_sender: w_tx.clone(),
        id_shared: swarm.behaviour_mut().id_store.shared.clone(),
    };
    let wallet_store = WalletStore::new(wallet_opt)?;
    wallet_store.create("ademcaglin", "123456").await?;
    wallet_store.logout().await;
    wallet_store.login("123456").await?;
    wallet_store.add_connection("abc", "abc").await;
    //let state = wallet_store.get_state()?;
    //let did = state.session.unwrap().payload.identity.clone();
    swarm.listen_on(format!("/ip4/0.0.0.0/tcp/43727").parse()?)?;
    //let topic = IdentTopic::new(&did.id);
    //swarm.behaviour_mut().gossipsub.subscribe(&topic)?;
    //swarm.behaviour_mut().id_store.push_did(did.clone());
    loop {
        tokio::select! {
            line = stdin.next_line() => {
                let line = line?.expect("stdin closed");
                run_command(&line, wallet_store.clone());
            }
            swarm_event = swarm.select_next_some() => {
                match swarm_event {
                    SwarmEvent::NewListenAddr { address, .. } => {
                        println!("Listening on {:?}", address);
                    }
                    SwarmEvent::Behaviour(IdentityNodeEvent::Gossipsub(event)) =>{
                        println!("Got message: {:?}", event);
                    }
                    other => {println!("{:?}", other);}
                }
            }
            event = rx.recv() => {
                if let Some(event) = event{
                    match event{
                        _ => {  }
                    }
                }
            }
        }
    }
}
