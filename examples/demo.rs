use colored::Colorize;
use idp2p_common::ed_secret::EdSecret;
use idp2p_core::did::Identity;
use idp2p_didcomm::jpm::Jpm;
use idp2p_didcomm::jwe::Jwe;
use idp2p_didcomm::jwm::Jwm;
use idp2p_didcomm::jws::Jws;
use idp2p_node::message::{IdentityMessage, IdentityMessagePayload};
use idp2p_node::node::build_gossipsub;
use idp2p_node::node::build_transport;
use idp2p_node::store::IdStore;
use idp2p_node::IdentityEvent;
use libp2p::Multiaddr;
use libp2p::{
    futures::StreamExt,
    gossipsub::{Gossipsub, GossipsubEvent, IdentTopic},
    identity::{ed25519::SecretKey, Keypair},
    mdns::{Mdns, MdnsEvent},
    swarm::{SwarmBuilder, SwarmEvent},
    NetworkBehaviour, PeerId,
};
use std::str::FromStr;
use structopt::StructOpt;
use tokio::io::AsyncBufReadExt;
use tokio::sync::mpsc::channel;

#[derive(Debug, StructOpt)]
#[structopt(name = "idp2p", about = "Usage of idp2p.")]
struct Opt {
    #[structopt(short = "i", long = "ip", default_value = "0.0.0.0")]
    ip: String,
    #[structopt(short = "p", long = "port", default_value = "43727")]
    port: u16,
    #[structopt(short = "r", long = "remote")]
    remote: Option<String>,
}

#[derive(NetworkBehaviour)]
#[behaviour(out_event = "IdentityNodeEvent")]
pub struct IdentityNodeBehaviour {
    mdns: Mdns,
    pub gossipsub: Gossipsub,
    #[behaviour(ignore)]
    pub id_store: IdStore,
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
                        self.id_store.handle_jwm(&topic, message).await;
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
pub fn handle_jwm(jwm: &str, behaviour: &mut IdentityNodeBehaviour) {
    let secret_str = std::env::var("secret").unwrap();
    let dec_secret = EdSecret::from_str(&secret_str).unwrap();
    let jwe: Jwe = idp2p_common::serde_json::from_str(jwm).unwrap();
    let json = jwe.decrypt(dec_secret).unwrap();
    let jws: Jws = idp2p_common::serde_json::from_str(&json).unwrap();
    let jpm: Jpm = idp2p_common::base64url::decode(&jws.payload).unwrap();
    let from = behaviour.id_store.get_did(&jpm.from);
    jws.verify(from).unwrap();
    let rec_mes = format!("Received message {}", jpm.body.to_string().green());
    println!("{rec_mes}");
}

fn run_command(input: &str, behaviour: &mut IdentityNodeBehaviour) {
    let split = input.split(" ");
    let input: Vec<&str> = split.collect();
    match input[0] {
        "get" => {
            let topic = IdentTopic::new(input[1]);
            behaviour.gossipsub.subscribe(&topic).unwrap();
            let message = IdentityMessage::new(IdentityMessagePayload::Get);
            let data = idp2p_common::serde_json::to_vec(&message).unwrap();
            behaviour.gossipsub.publish(topic, data).unwrap();
        }
        "resolve" => {
            let did = behaviour.id_store.get_did(input[1]);
            println!("Resolved: {:?}", did);
        }
        "send-message" => {
            let secret_str = std::env::var("secret").unwrap();
            let secret = EdSecret::from_str(&secret_str).unwrap();
            let from_did = behaviour.id_store.get_owner();
            let to_did = behaviour.id_store.get_did(input[1]);
            let jwm = Jwm::new(from_did.clone(), to_did, input[2]);
            let jwe = jwm.seal(secret).unwrap();
            let json = idp2p_common::serde_json::to_string(&jwe).unwrap();
            let mes_payload = IdentityMessagePayload::Jwm { message: json };
            let mes = IdentityMessage::new(mes_payload);
            let topic = IdentTopic::new(input[1]);
            let data = idp2p_common::serde_json::to_vec(&mes).unwrap();
            behaviour.gossipsub.publish(topic, data).unwrap();
        }
        _ => {}
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let opt = Opt::from_args();
    let mut stdin = tokio::io::BufReader::new(tokio::io::stdin()).lines();
    let (tx, mut rx) = channel::<IdentityEvent>(100);
    let secret = EdSecret::new();
    let did = Identity::from_secret(secret.clone());
    std::env::set_var("secret", idp2p_common::encode(&secret.to_bytes()));
    println!("Created: {}", did.id);
    let secret_key = SecretKey::from_bytes(secret.to_bytes())?;
    let local_key = Keypair::Ed25519(secret_key.into());
    let transport = build_transport(local_key.clone()).await;
    let mut swarm = {
        let mdns = Mdns::new(Default::default()).await?;
        let id_store = IdStore::new(tx.clone(), did.clone());
        let behaviour = IdentityNodeBehaviour {
            mdns: mdns,
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
    swarm.listen_on(format!("/ip4/{}/tcp/{}", opt.ip, opt.port).parse()?)?;
    let topic = IdentTopic::new(&did.id);
    swarm.behaviour_mut().gossipsub.subscribe(&topic)?;
    swarm.behaviour_mut().id_store.push_did(did);
    if let Some(remote) = opt.remote {
        let vec = remote.split("/").collect::<Vec<&str>>();
        let to_dial = format!("/ip4/{}/tcp/{}", vec[2], vec[4]);
        let addr: Multiaddr = to_dial.parse().unwrap();
        let peer_id = PeerId::from_str(vec[6])?;
        swarm.dial(addr)?;
        swarm.behaviour_mut().gossipsub.add_explicit_peer(&peer_id);
    }
    loop {
        tokio::select! {
            line = stdin.next_line() => {
                let line = line?.expect("stdin closed");
                run_command(&line, swarm.behaviour_mut());
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
                    other => {println!("{:?}", other);}
                }
            }
            event = rx.recv() => {
                if let Some(event) = event{
                    match event{
                        IdentityEvent::Published { id } => { swarm.behaviour_mut().post(&id); }
                        IdentityEvent::ReceivedJwm { id, jwm  } => { handle_jwm(&jwm, swarm.behaviour_mut()) }
                        _ => {  }
                    }
                }
            }
        }
    }
}
