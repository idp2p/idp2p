use idp2p_common::{anyhow::Result, serde_json};
use idp2p_node::message::{IdentityMessage, IdentityMessagePayload};
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
    mdns::{Mdns, MdnsEvent},
    swarm::{SwarmBuilder, SwarmEvent},
    NetworkBehaviour,
};
use std::collections::HashMap;
use std::path::PathBuf;
use std::str::FromStr;
use std::sync::Arc;
use tokio::io::AsyncBufReadExt;
use tokio::sync::mpsc::channel;

#[derive(NetworkBehaviour)]
#[behaviour(out_event = "IdentityNodeEvent")]
pub struct IdentityNodeBehaviour {
    mdns: Mdns,
    pub gossipsub: Gossipsub,
    #[behaviour(ignore)]
    pub id_store: Arc<IdStore>,
    #[behaviour(ignore)]
    pub wallet_store: Option<Arc<WalletStore>>,
}

#[derive(Debug)]
pub enum IdentityNodeEvent {
    Mdns(MdnsEvent),
    Gossipsub(GossipsubEvent),
}

impl From<MdnsEvent> for IdentityNodeEvent {
    fn from(event: MdnsEvent) -> Self {
        IdentityNodeEvent::Mdns(event)
    }
}
impl From<GossipsubEvent> for IdentityNodeEvent {
    fn from(event: GossipsubEvent) -> Self {
        IdentityNodeEvent::Gossipsub(event)
    }
}

impl IdentityNodeBehaviour {
    pub async fn handle_gossipevent(&mut self, event: GossipsubEvent) {
        if let GossipsubEvent::Message {
            propagation_source: _,
            message_id: _,
            message,
        } = event
        {
            let topic = message.topic.to_string();
            if idp2p_common::is_idp2p(&topic) {
                let message = IdentityMessage::from_bytes(&message.data);
                match &message.payload {
                    IdentityMessagePayload::Get => {
                        self.id_store.handle_get(&topic).await;
                    }
                    IdentityMessagePayload::Post { digest, identity } => {
                        self.id_store.handle_post(digest, identity).await.unwrap();
                    }
                    IdentityMessagePayload::Jwm { message } => {
                        //handle_jwm(&message, self);
                    }
                }
            }
        }
    }

    pub fn handle_mdnsevent(&mut self, event: MdnsEvent) {
        match event {
            MdnsEvent::Discovered(list) => {
                for (peer, _) in list {
                    self.gossipsub.add_explicit_peer(&peer);
                }
            }
            MdnsEvent::Expired(list) => {
                for (peer, _) in list {
                    if !self.mdns.has_node(&peer) {
                        self.gossipsub.remove_explicit_peer(&peer);
                    }
                }
            }
        }
    }

    pub fn post(&mut self, id: &str) {
        let did = self.id_store.get_did(id);
        let gossip_topic = IdentTopic::new(id);
        let message = IdentityMessage::new_post(did);
        let json_str = idp2p_common::serde_json::to_string(&message).unwrap();
        let result = self.gossipsub.publish(gossip_topic, json_str.as_bytes());
        match result {
            Ok(_) => println!("Published id: {}", id),
            Err(e) => println!("Publish error, {:?}", e),
        }
    }
}

async fn run_command(input: Vec<&str>, behaviour: &mut IdentityNodeBehaviour) -> Result<()> {
   if let Some(ref mut store) = behaviour.wallet_store {
        match input[0] {
            "get" => {
                let state = serde_json::to_string_pretty(&store.get_state()?)?;
                println!("{state}");
            }
            "create" => {
                store.register(input[1], input[2]).await?;
                let state = store.get_state()?;
                println!("Id: {}", state.session_wallet.unwrap().identity.id);
            }
            "login" => {
                store.login(input[1]).await?;
            }
            "logout" => {
                store.logout().await;
            }
            "connect" => {
                store.connect(input[1], input[2]).await?;
            }
            "send-message" => {
                store
                    .send_message(behaviour.id_store.clone(), input[1], input[2])
                    .await?;
            }
            _ => {}
        }
    }
    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut stdin = tokio::io::BufReader::new(tokio::io::stdin()).lines();
    let (tx, mut rx) = channel::<IdentityEvent>(100);
    let (wallet_tx, mut wallet_rx) = channel::<WalletEvent>(100);

    let local_key = Keypair::generate_ed25519();
    let transport = build_transport(local_key.clone()).await;
    let options = IdStoreOptions {
        event_sender: tx.clone(),
        entries: HashMap::new(),
    };
    let id_store = IdStore::new(options);
    let mdns = Mdns::new(Default::default()).await?;
    let mut swarm = {
        let behaviour = IdentityNodeBehaviour {
            mdns: mdns,
            gossipsub: build_gossipsub(),
            id_store: Arc::new(id_store),
            wallet_store: None,
        };
        let executor = Box::new(|fut| {
            tokio::spawn(fut);
        });
        SwarmBuilder::new(transport, behaviour, local_key.public().to_peer_id())
            .executor(executor)
            .build()
    };
    
    loop {
        tokio::select! {
            line = stdin.next_line() => {
                let line = line?.expect("stdin closed");
                let split = line.split(" ");
                let input: Vec<&str> = split.collect();
                if swarm.behaviour_mut().wallet_store.is_some() {
                    run_command(input, swarm.behaviour_mut()).await?;
                }
                else if input[0] == "listen"{
                    let base_path = format!("./target/{}", input[1]);
                    std::fs::create_dir_all(base_path.clone())?;
                    let wallet_opt = WalletOptions {
                        wallet_path: PathBuf::from_str(&format!("{base_path}/wallet.json"))?,
                        event_sender: wallet_tx.clone(),
                    };
                    let wallet_store = WalletStore::new(wallet_opt)?;
                    swarm.behaviour_mut().wallet_store = Some(Arc::new(wallet_store));
                    swarm.listen_on(format!("/ip4/0.0.0.0/tcp/{}", input[1]).parse()?)?;

                }
            }
            swarm_event = swarm.select_next_some() => {
                match swarm_event {
                    SwarmEvent::NewListenAddr { address, .. } => {
                        println!("Listening on {:?}", address);
                    }
                    SwarmEvent::Behaviour(IdentityNodeEvent::Gossipsub(event)) =>{
                        println!("Got message: {:?}", event);
                        swarm.behaviour_mut().handle_gossipevent(event).await;
                    }
                    SwarmEvent::Behaviour(IdentityNodeEvent::Mdns(event)) =>{
                        swarm.behaviour_mut().handle_mdnsevent(event);
                    }
                    _ => {}
                }
            }
            event = rx.recv() => {
                if let Some(event) = event{
                    match event{
                        IdentityEvent::Published { id } => { swarm.behaviour_mut().post(&id); }
                        _ => {  }
                    }
                }
            }
            event = wallet_rx.recv() => {
                if let Some(event) = event{
                    match event{
                        WalletEvent::Connected(id) => {
                            let topic = IdentTopic::new(id);
                            swarm.behaviour_mut().gossipsub.subscribe(&topic).unwrap();
                            let message = IdentityMessage::new(IdentityMessagePayload::Get);
                            let data = idp2p_common::serde_json::to_vec(&message).unwrap();
                            swarm.behaviour_mut().gossipsub.publish(topic, data).unwrap();
                        }
                        WalletEvent::Created(did) => {
                            let topic = IdentTopic::new(did.id.clone());
                            swarm.behaviour_mut().id_store.push_did(did.clone());
                            swarm.behaviour_mut().gossipsub.subscribe(&topic).unwrap();
                        }
                        _ => {  }
                    }
                }
            }
        }
    }
}
